use clap::Parser;
use slide_flow::{
    parser::{
        Cmd,
        SubCommands::{Add, Bib, Build, Index, Init, PreCommit, Version},
        VersionCommands,
    },
    project::Project,
    subcommand::{
        add::add,
        bib::update_bibliography,
        build::{
            build, build_html_commands, build_pdf_commands, build_pdf_latest_alias_commands,
            copy_images_html, copy_ipe_pdf,
        },
        index::put_index,
        init::init,
        pre_commit::{create_files, remove_cache},
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
        Add {
            name,
            secret,
            draft,
            type_,
        } => add(&project, name, secret, draft, type_.unwrap_or_default()),
        PreCommit => {
            // remove cache
            remove_cache(&project)?;
            // create files
            create_files(&project)
        }
        Index { dir, quiet } => {
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
        Bib { dir } => {
            let target_slide = project.get_slide(&dir)?;

            update_bibliography(target_slide)
        }
        Build {
            directories,
            concurrent,
        } => {
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

                let build_html_cmd = build_html_commands(&project, &target_slide);
                let build_pdf_cmd = build_pdf_commands(&project, &target_slide);
                let build_pdf_latest_alias_cmd =
                    build_pdf_latest_alias_commands(&project, &target_slide);

                // copy images
                if let Err(e) = copy_images_html(&project, &target_slide) {
                    log::error!("Failed to copy images: {}", e);
                    continue;
                }

                cmds.extend(build_html_cmd);
                cmds.extend(build_pdf_cmd);
                cmds.extend(build_pdf_latest_alias_cmd);

                for archived in archived_slides {
                    if archived.conf.type_.is_marp() {
                        cmds.extend(build_pdf_commands(&project, &archived));
                    }
                }
            }

            build(cmds.into_iter(), concurrent);

            Ok(())
        }
        Version { command } => match command {
            VersionCommands::Bump { dir } => bump(&project, dir),
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
