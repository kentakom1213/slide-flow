//! build slides locally

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use colored::Colorize;
use tokio::{process::Command, runtime::Runtime, sync::Semaphore};

use crate::{config::SlideConf, project::Project, slide::Slide};

/// build commands and their information
pub enum BuildCommand {
    /// build command for PDF
    PDF {
        /// target directory
        dir: PathBuf,
        /// build command
        command: Command,
        /// slide configuration
        conf: SlideConf,
    },
    /// build command for HTML
    HTML {
        /// target directory
        dir: PathBuf,
        /// build command
        command: Command,
        /// slide configuration
        conf: SlideConf,
    },
}

/// run build commands
pub fn build(commands: impl Iterator<Item = BuildCommand>, max_concurrent: usize) {
    // initialize tokio runtime
    let runtime = Runtime::new().unwrap();

    // run build commands parallelly
    runtime.block_on(async {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        let handles: Vec<_> = commands
            .into_iter()
            .filter(|cmd| match cmd {
                BuildCommand::HTML { conf, .. } => !conf.draft.unwrap_or(false),
                BuildCommand::PDF { conf, .. } => !conf.draft.unwrap_or(false),
            })
            .map(|cmd| {
                let (dir, build_type, mut command) = match cmd {
                    BuildCommand::PDF { dir, command, .. } => (dir, "PDF", command),
                    BuildCommand::HTML { dir, command, .. } => (dir, "HTML", command),
                };

                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire_owned().await.unwrap();

                    match command.output().await {
                        Ok(_) => {
                            log::info!("build {}: {:?} ... {}", build_type, dir, "done".green());
                        }
                        Err(e) => {
                            log::error!("build {}: {:?} ... {}", build_type, dir, "failed".red());
                            log::error!("error: {:?}", e);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }
    });
}

/// generate file stems for output files
fn make_file_stems(slide: &Slide) -> Vec<String> {
    let mut res = slide.conf.custom_path.clone().unwrap_or_default();

    if let Some(prefix) = &slide.conf.secret {
        res.push(prefix.clone());
    } else {
        res.push(slide.conf.name.clone());
    }

    res
}

/// generate build commands for PDF
pub fn build_pdf_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    // closure to generate build command
    let make_commmand = move |output_stem: String| {
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd
            // specify theme
            .arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            // enable html
            .arg("--html")
            .arg("true")
            // output name
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(output_stem)
                    .with_extension("pdf"),
            )
            // output format
            .arg("--pdf")
            .arg("--allow-local-files")
            // title
            .arg("--title")
            .arg(&slide.conf.name)
            // author
            .arg("--author")
            .arg(&project.conf.author)
            // description
            .arg("--description")
            .arg(slide.conf.description.clone().unwrap_or_default())
            // input markdown file
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    // generate output file names
    let output_files = make_file_stems(slide);

    output_files.into_iter().map(move |stem| BuildCommand::PDF {
        dir: slide.dir.clone(),
        command: make_commmand(stem),
        conf: slide.conf.clone(),
    })
}

/// generate build commands for HTML
pub fn build_html_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    // closure to generate build command
    let make_commmand = move |output_stem: String| {
        // create command
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd
            // specify theme
            .arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            // enable html
            .arg("--html")
            .arg("true")
            // output name
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(&output_stem)
                    .join("index.html"),
            )
            // title
            .arg("--title")
            .arg(&slide.conf.name)
            // author
            .arg("--author")
            .arg(&project.conf.author)
            // description
            .arg("--description")
            .arg(slide.conf.description.clone().unwrap_or_default())
            // input markdown file
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    // generate output file names
    let output_files = make_file_stems(slide);

    output_files
        .into_iter()
        .map(move |stem| BuildCommand::HTML {
            dir: slide.dir.clone(),
            command: make_commmand(stem),
            conf: slide.conf.clone(),
        })
}

/// copy images to output directory
pub fn copy_images_html<'a>(project: &'a Project, slide: &'a Slide) -> anyhow::Result<()> {
    let output_files = make_file_stems(slide);

    for stem in output_files {
        let target_images_dir = project
            .root_dir
            .join(&project.conf.output_dir)
            .join(&stem)
            .join("images");

        // create target directory
        if target_images_dir.exists() {
            std::fs::remove_dir_all(&target_images_dir)?;
        }
        std::fs::create_dir_all(&target_images_dir)?;

        // copy images
        copy_images(slide, &target_images_dir)?;
    }

    Ok(())
}

/// copy images
fn copy_images(slide: &Slide, target_images_dir: &Path) -> anyhow::Result<()> {
    let slide_images_dir = slide.image_dir();

    // if the slide does not have images, return
    if !slide_images_dir.exists() {
        return Ok(());
    }

    // copy image files
    let images = std::fs::read_dir(&slide_images_dir)?;

    for image in images.filter_map(|e| e.ok()) {
        let path = image.path();
        let file_name = path.file_name().unwrap();
        // skip hidden files
        if file_name.to_string_lossy().starts_with(".") {
            continue;
        }

        let save_path = target_images_dir.join(file_name);
        if save_path.exists() {
            anyhow::bail!("The image file already exists: {:?}", save_path);
        }

        std::fs::copy(&path, &save_path)?;
    }

    Ok(())
}
