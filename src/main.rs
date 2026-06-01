use clap::Parser;
use slide_flow::{
    config::PathStrategy,
    images::{clean_image_cache, optimize_slide_images, print_report, OptimizeOptions},
    parser::{
        Cmd, ImagesCommands, MigrateCommands, SlidesCommands,
        SubCommands::{Build, Images, Init, Migrate, Slide},
    },
    project::Project,
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
        slide::show,
        version::bump,
    },
};
use std::io::Write;

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
            directories,
            concurrent,
            no_optimize_images,
        } => {
            let optimize_options = OptimizeOptions {
                dry_run: false,
                force: false,
            };
            let optimize_images = !no_optimize_images;
            // generate build commands
            let mut cmds = vec![];

            for dir in directories {
                let Ok(target_slide) = project.get_slide(&dir) else {
                    log::error!("The slide does not exist: {}", dir.to_string_lossy());
                    continue;
                };

                let Ok(archived_slides) = project.get_archived_slides(&target_slide) else {
                    log::error!(
                        "Failed to load archived slide versions: {}",
                        dir.to_string_lossy()
                    );
                    continue;
                };

                if target_slide.conf.type_.is_ipe() {
                    if let Err(e) = copy_ipe_pdf(&project, &target_slide, true) {
                        log::error!("Failed to pdf: {}", e);
                    }
                    for archived in archived_slides {
                        if let Err(e) = copy_ipe_pdf(&project, &archived, false) {
                            log::error!("Failed to archived pdf: {}", e);
                        }
                    }
                    log::info!("Copy PDF: {}", dir.to_string_lossy());
                    continue;
                }

                // copy images
                if let Err(e) =
                    copy_images_html_with_options(&project, &target_slide, optimize_images)
                {
                    log::error!("Failed to copy images: {}", e);
                    continue;
                }

                let build_html_cmd = match build_html_commands_with_options(
                    &project,
                    &target_slide,
                    &optimize_options,
                    optimize_images,
                ) {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        log::error!("Failed to prepare HTML build: {}", e);
                        continue;
                    }
                };
                let build_pdf_cmd = match build_pdf_commands_with_options(
                    &project,
                    &target_slide,
                    &optimize_options,
                    optimize_images,
                ) {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        log::error!("Failed to prepare PDF build: {}", e);
                        continue;
                    }
                };
                let build_pdf_latest_alias_cmd = match build_pdf_latest_alias_commands_with_options(
                    &project,
                    &target_slide,
                    &optimize_options,
                    optimize_images,
                ) {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        log::error!("Failed to prepare latest PDF build: {}", e);
                        continue;
                    }
                };
                let build_ogp_image_cmd = match build_ogp_image_commands_with_options(
                    &project,
                    &target_slide,
                    &optimize_options,
                    optimize_images,
                ) {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        log::error!("Failed to prepare OGP image build: {}", e);
                        continue;
                    }
                };

                cmds.extend(build_html_cmd);
                cmds.extend(build_pdf_cmd);
                cmds.extend(build_pdf_latest_alias_cmd);
                cmds.extend(build_ogp_image_cmd);

                let root_path_strategy = project.path_strategy(&target_slide);
                let build_archived_html =
                    root_path_strategy == PathStrategy::CanonicalWithRedirects;

                for archived in &archived_slides {
                    let mut archived = archived.clone();
                    if root_path_strategy == PathStrategy::CanonicalWithRedirects {
                        archived.conf.path_strategy = Some(root_path_strategy);
                    }

                    if archived.conf.type_.is_marp() {
                        if build_archived_html {
                            if let Err(e) =
                                copy_images_html_with_options(&project, &archived, optimize_images)
                            {
                                log::error!(
                                    "Failed to copy archived images {}: {}",
                                    archived.dir.to_string_lossy(),
                                    e
                                );
                                continue;
                            }

                            match build_html_commands_with_options(
                                &project,
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
                                &project,
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
                            &project,
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

                if let Err(e) = write_alias_redirects(&project, &target_slide, &archived_slides) {
                    log::error!("Failed to write alias redirects: {}", e);
                }
            }

            build(cmds.into_iter(), concurrent);

            Ok(())
        }
        Images { command } => match command {
            ImagesCommands::Optimize {
                dir,
                dry_run,
                force,
            } => {
                let slide = project.get_slide(&dir)?;
                let report =
                    optimize_slide_images(&project, &slide, &OptimizeOptions { dry_run, force })?;
                print_report(&report, &project);
                Ok(())
            }
            ImagesCommands::OptimizeAll { dry_run, force } => {
                for slide in &project.slides {
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
                draft,
                type_,
            } => add(&project, name, secret, draft, type_.unwrap_or_default()),
            SlidesCommands::List => list(&project),
            SlidesCommands::Show { selector } => show(&project, &selector),
            SlidesCommands::Archive { dir } => bump(&project, dir),
            SlidesCommands::Index { dir, quiet } => {
                if let Some(dir) = dir {
                    let target_slide = project.get_slide(&dir)?;

                    let toc = put_index(&target_slide)?;

                    if !quiet {
                        println!("{toc}");
                    }

                    Ok(())
                } else {
                    project
                        .slides
                        .iter()
                        .inspect(|slide| {
                            log::info!("Put index to slide: {}", slide.dir.to_string_lossy())
                        })
                        .try_for_each(|slide| {
                            let _toc = put_index(slide)?;
                            Ok(())
                        })
                }
            }
            SlidesCommands::Bib { dir } => {
                let target_slide = project.get_slide(&dir)?;

                update_bibliography(target_slide)
            }
        },
    }
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
