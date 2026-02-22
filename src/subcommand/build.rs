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

/// generate file stems for output files
pub fn make_file_stems(slide: &Slide) -> Vec<String> {
    let mut res = slide.conf.custom_path.clone().unwrap_or_default();

    if let Some(prefix) = &slide.conf.secret {
        res.push(prefix.clone());
    } else {
        res.push(slide.conf.name.clone());
    }

    res
}

/// append `_v<version>` to file stems and use them as published URLs
pub fn make_versioned_stems(slide: &Slide) -> Vec<String> {
    make_file_stems(slide)
        .into_iter()
        .map(|stem| format!("{stem}_v{}", slide.conf.version))
        .collect()
}

/// latest PDF aliases (`<stem>.pdf`) for current slide
pub fn make_latest_pdf_aliases(slide: &Slide) -> Vec<String> {
    make_file_stems(slide)
        .into_iter()
        .map(|stem| format!("{stem}.pdf"))
        .collect()
}

/// generate build commands for PDF
pub fn build_pdf_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    let make_command = move |output_stem: String| {
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd.arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            .arg("--html")
            .arg("true")
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(output_stem)
                    .with_extension("pdf"),
            )
            .arg("--pdf")
            .arg("--allow-local-files")
            .arg("--title")
            .arg(&slide.conf.name)
            .arg("--author")
            .arg(&project.conf.author)
            .arg("--description")
            .arg(slide.conf.description.clone().unwrap_or_default())
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    make_versioned_stems(slide)
        .into_iter()
        .map(move |stem| BuildCommand::PDF {
            dir: slide.dir.clone(),
            command: make_command(stem),
            conf: slide.conf.clone(),
        })
}

/// generate build commands for latest PDF aliases (`<stem>.pdf`)
pub fn build_pdf_latest_alias_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    let make_command = move |output_file_name: String| {
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd.arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            .arg("--html")
            .arg("true")
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(output_file_name),
            )
            .arg("--pdf")
            .arg("--allow-local-files")
            .arg("--title")
            .arg(&slide.conf.name)
            .arg("--author")
            .arg(&project.conf.author)
            .arg("--description")
            .arg(slide.conf.description.clone().unwrap_or_default())
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    make_latest_pdf_aliases(slide)
        .into_iter()
        .map(move |file_name| BuildCommand::PDF {
            dir: slide.dir.clone(),
            command: make_command(file_name),
            conf: slide.conf.clone(),
        })
}

/// generate build commands for HTML
pub fn build_html_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    let make_command = move |output_stem: String| {
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd.arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            .arg("--html")
            .arg("true")
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(&output_stem)
                    .join("index.html"),
            )
            .arg("--title")
            .arg(&slide.conf.name)
            .arg("--author")
            .arg(&project.conf.author)
            .arg("--description")
            .arg(slide.conf.description.clone().unwrap_or_default())
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    make_versioned_stems(slide)
        .into_iter()
        .map(move |stem| BuildCommand::HTML {
            dir: slide.dir.clone(),
            command: make_command(stem),
            conf: slide.conf.clone(),
        })
}

/// copy ipe slide pdf to output directory
pub fn copy_ipe_pdf(
    project: &Project,
    slide: &Slide,
    include_latest_alias: bool,
) -> anyhow::Result<()> {
    let source_pdf_path = slide.dir.join("slide.pdf");

    for stem in make_versioned_stems(slide) {
        let pdf_save_path = project
            .root_dir
            .join(&project.conf.output_dir)
            .join(stem + ".pdf");

        std::fs::copy(&source_pdf_path, &pdf_save_path)?;
    }

    if include_latest_alias {
        for file_name in make_latest_pdf_aliases(slide) {
            let pdf_save_path = project
                .root_dir
                .join(&project.conf.output_dir)
                .join(file_name);
            std::fs::copy(&source_pdf_path, &pdf_save_path)?;
        }
    }

    Ok(())
}

/// copy images to output directory
pub fn copy_images_html(project: &Project, slide: &Slide) -> anyhow::Result<()> {
    for stem in make_versioned_stems(slide) {
        let target_images_dir = project
            .root_dir
            .join(&project.conf.output_dir)
            .join(&stem)
            .join("images");

        if target_images_dir.exists() {
            std::fs::remove_dir_all(&target_images_dir)?;
        }
        std::fs::create_dir_all(&target_images_dir)?;

        copy_images(slide, &target_images_dir)?;
    }

    Ok(())
}

/// copy images
fn copy_images(slide: &Slide, target_images_dir: &Path) -> anyhow::Result<()> {
    let slide_images_dir = slide.image_dir();

    if !slide_images_dir.exists() {
        return Ok(());
    }

    let images = std::fs::read_dir(&slide_images_dir)?;

    for image in images.filter_map(|e| e.ok()) {
        let path = image.path();
        let file_name = path.file_name().unwrap();

        if file_name.to_string_lossy().starts_with('.') {
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
