use clap::Parser;
use slide_flow::{
    parser::{
        Cmd,
        SubCommands::{Add, Build, Index, Init, PreCommit},
    },
    project::Project,
    subcommand::{
        add::add,
        build::{build, build_html_commands, build_pdf_commands, copy_images_html},
        index::put_index,
        init::init,
        pre_commit::{create_files, remove_cache},
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
        } => add(&project, name, secret, draft),
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
                    .inspect(|slide| log::info!("Put index to slide: {:?}", slide.dir))
                    .map(|slide| put_index(slide).map(|_| ()))
                    .collect::<anyhow::Result<()>>()
            }
        }
        Build {
            directories,
            concurrent,
        } => {
            // generate build commands
            let mut cmds = vec![];

            for dir in directories {
                let Ok(target_slide) = project.get_slide(&dir) else {
                    log::error!("The slide does not exist: {:?}", &dir);
                    continue;
                };

                let build_html_cmd = build_html_commands(&project, &target_slide);
                let build_pdf_cmd = build_pdf_commands(&project, &target_slide);

                // copy images
                if let Err(e) = copy_images_html(&project, &target_slide) {
                    log::error!("Failed to copy images: {:?}", e);
                    continue;
                }

                cmds.extend(build_html_cmd);
                cmds.extend(build_pdf_cmd);
            }

            build(cmds.into_iter(), concurrent);

            Ok(())
        }
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
