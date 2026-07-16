use clap::Parser;
use slide_flow::{
    config::PathStrategy,
    images::{clean_image_cache, optimize_slide_images, print_report, OptimizeOptions},
    parser::{
        Cmd, ImagesCommands, MigrateCommands, OptionalTargetArgs, ProjectCommands, PruneCommands,
        RequiredTargetArgs, SlidesCommands,
        SubCommands::{
            Bib, Build, Images, Init, Migrate, Prepare, Project as ProjectCmd, Prune, Slide, Toc,
        },
    },
    project::Project,
    slide::Slide as SlideData,
    subcommand::{
        add::add,
        bib::update_bibliography,
        build::{
            build, build_html_commands_with_options, build_ogp_image_commands_with_options,
            build_pdf_commands_with_options, build_pdf_latest_alias_commands_with_options,
            copy_images_html_with_options, copy_ipe_pdf, write_alias_redirects,
        },
        index::put_index,
        init::init,
        list::list,
        migrate::{apply, plan, status, ApplyOptions},
        pre_commit::{prune_stale_outputs, refresh_project_files},
        slide::show,
        version::bump,
    },
};
use std::{
    collections::BTreeSet,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

fn init_logger() {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}] {}", record.level(), record.args()))
        .filter(None, log::LevelFilter::Trace)
        .init();
}

fn runner() -> anyhow::Result<()> {
    // initialize logger
    init_logger();

    // get current directory
    let root_dir = std::env::current_dir()?;

    // parse command line arguments
    let parser = Cmd::parse();

    // get project information
    let project = Project::get(root_dir.clone());

    if matches!(parser.subcommand, Init) {
        // if init command, check if the project already exists
        if project.is_ok() {
            log::error!("The project already exists.");
        } else {
            // if not, create a new project
            init(&root_dir)?;
        }
        return Ok(());
    }

    let project = project?;

    // run subcommand
    match parser.subcommand {
        Init => unreachable!(),
        Build {
            targets,
            concurrent,
            no_optimize_images,
        } => {
            let slides = resolve_required_targets(&project, &targets)?;
            build_slides(&project, &slides, concurrent, !no_optimize_images);
            Ok(())
        }
        Prepare {
            targets,
            no_refresh,
            no_toc,
            no_bib,
            no_build,
            no_optimize_images,
            concurrent,
            dry_run,
        } => {
            let slides = resolve_optional_targets_default_changed(&project, &targets)?;
            prepare(
                &project,
                &slides,
                PrepareOptions {
                    refresh: !no_refresh,
                    toc: !no_toc,
                    bib: !no_bib,
                    build: !no_build,
                    optimize_images: !no_optimize_images,
                    concurrent,
                    dry_run,
                },
            )
        }
        Toc { targets, quiet } => {
            let slides = resolve_required_targets(&project, &targets)?;
            update_toc(&slides, quiet)
        }
        Bib { targets } => {
            let slides = resolve_required_targets(&project, &targets)?;
            update_bib(&slides)
        }
        Prune { command } => match command {
            PruneCommands::Outputs { dry_run, apply } => {
                prune_stale_outputs(&project, apply && !dry_run)
            }
        },
        ProjectCmd { command } => match command {
            ProjectCommands::List => list(&project),
            ProjectCommands::Show => show_project(&project),
            ProjectCommands::Refresh => refresh_project_files(&project),
        },
        Images { command } => match command {
            ImagesCommands::Optimize {
                targets,
                dry_run,
                force,
            } => {
                let slides = resolve_required_targets(&project, &targets)?;
                for slide in &slides {
                    let report = optimize_slide_images(
                        &project,
                        slide,
                        &OptimizeOptions { dry_run, force },
                    )?;
                    print_report(&report, &project);
                }
                Ok(())
            }
            ImagesCommands::Clean => {
                let cache_dir = clean_image_cache(&project)?;
                println!("Removed image cache: {}", cache_dir.to_string_lossy());
                Ok(())
            }
        },
        Migrate { command } => match command {
            MigrateCommands::Plan { dir } => plan(&project, dir),
            MigrateCommands::Status => status(&project),
            MigrateCommands::Apply {
                dir,
                metadata_only,
                redirects_only,
                artifacts,
                remove_legacy_artifacts,
                concurrent,
            } => apply(
                &project,
                dir,
                ApplyOptions {
                    metadata_only,
                    redirects_only,
                    artifacts,
                    remove_legacy_artifacts,
                    concurrent,
                },
            ),
        },
        Slide { command } => match command {
            SlidesCommands::Add {
                name,
                secret,
                public,
                draft,
                type_,
            } => add(
                &project,
                name,
                secret || !public,
                draft,
                type_.unwrap_or_default(),
            ),
            SlidesCommands::Show { selector } => show(&project, &selector),
            SlidesCommands::Archive { dir } => bump(&project, dir),
        },
    }
}

struct PrepareOptions {
    refresh: bool,
    toc: bool,
    bib: bool,
    build: bool,
    optimize_images: bool,
    concurrent: usize,
    dry_run: bool,
}

fn resolve_required_targets(
    project: &Project,
    targets: &RequiredTargetArgs,
) -> anyhow::Result<Vec<SlideData>> {
    if targets.all {
        return Ok(project.slides.clone());
    }

    if targets.changed {
        return changed_slides(project);
    }

    explicit_slides(project, &targets.directories)
}

fn resolve_optional_targets_default_changed(
    project: &Project,
    targets: &OptionalTargetArgs,
) -> anyhow::Result<Vec<SlideData>> {
    if targets.all {
        return Ok(project.slides.clone());
    }

    if targets.changed || targets.directories.is_empty() {
        return changed_slides(project);
    }

    explicit_slides(project, &targets.directories)
}

fn explicit_slides(project: &Project, directories: &[PathBuf]) -> anyhow::Result<Vec<SlideData>> {
    directories
        .iter()
        .map(|dir| project.get_slide(dir))
        .collect::<anyhow::Result<Vec<_>>>()
}

fn changed_slides(project: &Project) -> anyhow::Result<Vec<SlideData>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(&project.root_dir)
        .arg("status")
        .arg("--porcelain")
        .arg("--")
        .arg("src")
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git status: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "failed to detect changed slides with git: {}",
            stderr.trim()
        );
    }

    let stdout = String::from_utf8(output.stdout)?;
    let mut dirs = BTreeSet::new();

    for line in stdout.lines() {
        let Some(path) = line.get(3..) else {
            continue;
        };
        let path = path
            .rsplit_once(" -> ")
            .map(|(_, new_path)| new_path)
            .unwrap_or(path)
            .trim();
        let path = Path::new(path);
        let mut components = path.components();

        if components.next().and_then(|c| c.as_os_str().to_str()) != Some("src") {
            continue;
        }

        let Some(slide_name) = components.next() else {
            continue;
        };

        dirs.insert(PathBuf::from("src").join(slide_name.as_os_str()));
    }

    dirs.into_iter()
        .filter_map(|dir| match project.get_slide(&dir) {
            Ok(slide) => Some(Ok(slide)),
            Err(e) => {
                log::warn!(
                    "skip changed path without managed slide {}: {}",
                    dir.display(),
                    e
                );
                None
            }
        })
        .collect()
}

fn update_toc(slides: &[SlideData], quiet: bool) -> anyhow::Result<()> {
    for slide in slides {
        log::info!("Put index to slide: {}", slide.dir.to_string_lossy());
        let toc = put_index(slide)?;
        if !quiet {
            println!("{toc}");
        }
    }

    Ok(())
}

fn update_bib(slides: &[SlideData]) -> anyhow::Result<()> {
    for slide in slides {
        update_bibliography(slide.clone())?;
    }

    Ok(())
}

fn prepare(project: &Project, slides: &[SlideData], options: PrepareOptions) -> anyhow::Result<()> {
    if options.dry_run {
        println!("Targets:");
        for slide in slides {
            println!("- {}", display_project_path(project, &slide.dir));
        }

        println!("Planned steps:");
        print_planned_step(options.refresh, "project refresh");
        print_planned_step(options.toc, "toc");
        print_planned_step(options.bib, "bib");
        print_planned_step(options.build, "build");
        print_planned_step(options.build, "prune stale outputs");
        return Ok(());
    }

    if options.refresh {
        refresh_project_files(project)?;
    }
    if options.toc {
        update_toc(slides, true)?;
    }
    if options.bib {
        update_bib(slides)?;
    }
    if options.build {
        build_slides(project, slides, options.concurrent, options.optimize_images);
        prune_stale_outputs(project, true)?;
    }

    Ok(())
}

fn print_planned_step(enabled: bool, label: &str) {
    if enabled {
        println!("- {label}");
    }
}

fn show_project(project: &Project) -> anyhow::Result<()> {
    println!("Project root: {}", project.root_dir.to_string_lossy());
    println!(
        "Output directory: {}",
        project
            .root_dir
            .join(&project.conf.output_dir)
            .to_string_lossy()
    );
    println!(
        "Source directory: {}",
        project.root_dir.join("src").to_string_lossy()
    );
    println!("Managed slides: {}", project.slides.len());
    Ok(())
}

fn display_project_path(project: &Project, path: &Path) -> String {
    path.strip_prefix(&project.root_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn build_slides(project: &Project, slides: &[SlideData], concurrent: usize, optimize_images: bool) {
    let optimize_options = OptimizeOptions {
        dry_run: false,
        force: false,
    };
    let mut cmds = vec![];

    for target_slide in slides {
        let Ok(archived_slides) = project.get_archived_slides(target_slide) else {
            log::error!(
                "Failed to load archived slide versions: {}",
                target_slide.dir.to_string_lossy()
            );
            continue;
        };

        if target_slide.conf.type_.is_ipe() {
            if let Err(e) = copy_ipe_pdf(project, target_slide, true) {
                log::error!("Failed to pdf: {}", e);
            }
            for archived in archived_slides {
                if let Err(e) = copy_ipe_pdf(project, &archived, false) {
                    log::error!("Failed to archived pdf: {}", e);
                }
            }
            log::info!("Copy PDF: {}", target_slide.dir.to_string_lossy());
            continue;
        }

        if let Err(e) = copy_images_html_with_options(project, target_slide, optimize_images) {
            log::error!("Failed to copy images: {}", e);
            continue;
        }

        match build_html_commands_with_options(
            project,
            target_slide,
            &optimize_options,
            optimize_images,
        ) {
            Ok(build_cmds) => cmds.extend(build_cmds),
            Err(e) => {
                log::error!("Failed to prepare HTML build: {}", e);
                continue;
            }
        }

        match build_pdf_commands_with_options(
            project,
            target_slide,
            &optimize_options,
            optimize_images,
        ) {
            Ok(build_cmds) => cmds.extend(build_cmds),
            Err(e) => {
                log::error!("Failed to prepare PDF build: {}", e);
                continue;
            }
        }

        match build_pdf_latest_alias_commands_with_options(
            project,
            target_slide,
            &optimize_options,
            optimize_images,
        ) {
            Ok(build_cmds) => cmds.extend(build_cmds),
            Err(e) => {
                log::error!("Failed to prepare latest PDF build: {}", e);
                continue;
            }
        }

        match build_ogp_image_commands_with_options(
            project,
            target_slide,
            &optimize_options,
            optimize_images,
        ) {
            Ok(build_cmds) => cmds.extend(build_cmds),
            Err(e) => {
                log::error!("Failed to prepare OGP image build: {}", e);
                continue;
            }
        }

        let root_path_strategy = project.path_strategy(target_slide);
        let build_archived_html = root_path_strategy == PathStrategy::CanonicalWithRedirects;

        for archived in &archived_slides {
            let mut archived = archived.clone();
            if root_path_strategy == PathStrategy::CanonicalWithRedirects {
                archived.conf.path_strategy = Some(root_path_strategy);
            }

            if archived.conf.type_.is_marp() {
                if build_archived_html {
                    if let Err(e) =
                        copy_images_html_with_options(project, &archived, optimize_images)
                    {
                        log::error!(
                            "Failed to copy archived images {}: {}",
                            archived.dir.to_string_lossy(),
                            e
                        );
                        continue;
                    }

                    match build_html_commands_with_options(
                        project,
                        &archived,
                        &optimize_options,
                        optimize_images,
                    ) {
                        Ok(archived_cmds) => cmds.extend(archived_cmds),
                        Err(e) => {
                            log::error!(
                                "Failed to prepare archived HTML build {}: {}",
                                archived.dir.to_string_lossy(),
                                e
                            );
                        }
                    }

                    match build_ogp_image_commands_with_options(
                        project,
                        &archived,
                        &optimize_options,
                        optimize_images,
                    ) {
                        Ok(archived_cmds) => cmds.extend(archived_cmds),
                        Err(e) => {
                            log::error!(
                                "Failed to prepare archived OGP image build {}: {}",
                                archived.dir.to_string_lossy(),
                                e
                            );
                        }
                    }
                }

                match build_pdf_commands_with_options(
                    project,
                    &archived,
                    &optimize_options,
                    optimize_images,
                ) {
                    Ok(archived_cmds) => cmds.extend(archived_cmds),
                    Err(e) => {
                        log::error!(
                            "Failed to prepare archived PDF build {}: {}",
                            archived.dir.to_string_lossy(),
                            e
                        );
                    }
                }
            }
        }

        if let Err(e) = write_alias_redirects(project, target_slide, &archived_slides) {
            log::error!("Failed to write alias redirects: {}", e);
        }
    }

    build(cmds.into_iter(), concurrent);
}

fn main() {
    // main function
    let res = runner();

    // error handling
    if let Err(e) = res {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
