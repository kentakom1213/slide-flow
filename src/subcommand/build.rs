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
                            log::info!(
                                "build {}: {} ... {}",
                                build_type,
                                dir.to_string_lossy(),
                                "done".green()
                            );
                        }
                        Err(e) => {
                            log::error!(
                                "build {}: {} ... {}",
                                build_type,
                                dir.to_string_lossy(),
                                "failed".red()
                            );
                            log::error!("error: {}", e);
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

pub fn pdf_file_name(slide_name: &str, version: u8) -> String {
    format!("{slide_name}_v{version}.pdf")
}

/// generate a build command for PDF
pub fn build_pdf_command(
    project: &Project,
    slide: &Slide,
    output_file_name: String,
) -> BuildCommand {
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
                .join(output_file_name),
        )
        // output format
        .arg("--pdf")
        .arg("--allow-local-files")
        // title
        .arg(&slide.conf.name)
        // author
        .arg("--author")
        .arg(&project.conf.author)
        // description
        .arg("--description")
        .arg(slide.conf.description.clone().unwrap_or_default())
        // input markdown file
        .arg(slide.dir.join("slide.md"));

    BuildCommand::PDF {
        dir: slide.dir.clone(),
        command: cmd,
        conf: slide.conf.clone(),
    }
}

/// generate a build command for HTML (latest only)
pub fn build_html_command(project: &Project, slide: &Slide) -> BuildCommand {
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
                .join(&slide.conf.name)
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

    BuildCommand::HTML {
        dir: slide.dir.clone(),
        command: cmd,
        conf: slide.conf.clone(),
    }
}

/// copy slide pdf to output directory
pub fn copy_ipe_pdf(
    project: &Project,
    slide: &Slide,
    output_file_name: &str,
) -> anyhow::Result<()> {
    let source_pdf_path = slide.dir.join("slide.pdf");
    let pdf_save_path = project
        .root_dir
        .join(&project.conf.output_dir)
        .join(output_file_name);
    std::fs::copy(&source_pdf_path, &pdf_save_path)?;

    Ok(())
}

/// copy images to output directory
pub fn copy_images_html(project: &Project, slide: &Slide) -> anyhow::Result<()> {
    let target_images_dir = project
        .root_dir
        .join(&project.conf.output_dir)
        .join(&slide.conf.name)
        .join("images");

    // create target directory
    if target_images_dir.exists() {
        std::fs::remove_dir_all(&target_images_dir)?;
    }
    std::fs::create_dir_all(&target_images_dir)?;

    // copy images
    copy_images(slide, &target_images_dir)?;

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
            anyhow::bail!(
                "The image file already exists: {}",
                save_path.to_string_lossy()
            );
        }

        std::fs::copy(&path, &save_path)?;
    }

    Ok(())
}
