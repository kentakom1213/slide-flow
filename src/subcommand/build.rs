//! build slides locally

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use colored::Colorize;
use tokio::{process::Command, runtime::Runtime, sync::Semaphore};

use crate::{
    config::{PathStrategy, SlideConf},
    path::{legacy_file_stems, PublishPlan},
    project::Project,
    slide::Slide,
};

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
        /// temporary marp input file to delete after build
        temp_input: Option<PathBuf>,
    },
    /// build command for HTML
    HTML {
        /// target directory
        dir: PathBuf,
        /// build command
        command: Command,
        /// slide configuration
        conf: SlideConf,
        /// temporary marp input file to delete after build
        temp_input: Option<PathBuf>,
    },
    /// build command for OGP image
    OGPImage {
        /// target directory
        dir: PathBuf,
        /// build command
        command: Command,
        /// slide configuration
        conf: SlideConf,
        /// temporary marp input file to delete after build
        temp_input: Option<PathBuf>,
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
                BuildCommand::OGPImage { conf, .. } => !conf.draft.unwrap_or(false),
            })
            .map(|cmd| {
                let (dir, build_type, mut command, temp_input) = match cmd {
                    BuildCommand::PDF {
                        dir,
                        command,
                        temp_input,
                        ..
                    } => (dir, "PDF", command, temp_input),
                    BuildCommand::HTML {
                        dir,
                        command,
                        temp_input,
                        ..
                    } => (dir, "HTML", command, temp_input),
                    BuildCommand::OGPImage {
                        dir,
                        command,
                        temp_input,
                        ..
                    } => (dir, "OGP", command, temp_input),
                };

                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire_owned().await.unwrap();

                    let output = command.output().await;

                    if let Some(path) = temp_input {
                        if let Err(e) = fs::remove_file(&path) {
                            log::warn!(
                                "failed to remove temp input {}: {}",
                                path.to_string_lossy(),
                                e
                            );
                        }
                    }

                    match output {
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

fn prepare_marp_input(
    project: &Project,
    slide: &Slide,
) -> anyhow::Result<(PathBuf, Option<PathBuf>)> {
    let original_path = slide.dir.join("slide.md");
    let suffix = project.conf.template.suffix.trim_end();

    if suffix.is_empty() {
        return Ok((original_path, None));
    }

    let mut contents = fs::read_to_string(&original_path)?;
    contents.push_str("\n\n");
    contents.push_str(suffix);
    contents.push('\n');

    let temp_path = slide
        .dir
        .join(format!(".slide-flow-build-{}.md", uuid::Uuid::new_v4()));
    fs::write(&temp_path, contents)?;

    Ok((temp_path.clone(), Some(temp_path)))
}

fn cleanup_temp_input(index: usize, len: usize, temp_input: &Option<PathBuf>) -> Option<PathBuf> {
    if index + 1 == len {
        temp_input.clone()
    } else {
        None
    }
}

/// generate file stems for output files
pub fn make_file_stems(slide: &Slide) -> Vec<String> {
    legacy_file_stems(slide)
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
) -> anyhow::Result<Vec<BuildCommand>> {
    let output_stems = PublishPlan::for_slide(project, slide).versioned_pdf_stems;
    if output_stems.is_empty() {
        return Ok(vec![]);
    }

    let (input_path, temp_input) = prepare_marp_input(project, slide)?;
    let make_command = |output_stem: String| {
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
            .arg(&input_path);

        cmd
    };

    let len = output_stems.len();

    Ok(output_stems
        .into_iter()
        .enumerate()
        .map(|(index, stem)| BuildCommand::PDF {
            dir: slide.dir.clone(),
            command: make_command(stem),
            conf: slide.conf.clone(),
            temp_input: cleanup_temp_input(index, len, &temp_input),
        })
        .collect())
}

/// generate build commands for latest PDF aliases (`<stem>.pdf`)
pub fn build_pdf_latest_alias_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> anyhow::Result<Vec<BuildCommand>> {
    let output_files = PublishPlan::for_slide(project, slide).latest_pdf_aliases;
    if output_files.is_empty() {
        return Ok(vec![]);
    }

    let (input_path, temp_input) = prepare_marp_input(project, slide)?;
    let make_command = |output_file_name: String| {
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
            .arg(&input_path);

        cmd
    };

    let len = output_files.len();

    Ok(output_files
        .into_iter()
        .enumerate()
        .map(|(index, file_name)| BuildCommand::PDF {
            dir: slide.dir.clone(),
            command: make_command(file_name),
            conf: slide.conf.clone(),
            temp_input: cleanup_temp_input(index, len, &temp_input),
        })
        .collect())
}

/// generate build commands for HTML
pub fn build_html_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> anyhow::Result<Vec<BuildCommand>> {
    let output_paths = PublishPlan::for_slide(project, slide).html_paths;
    if output_paths.is_empty() {
        return Ok(vec![]);
    }

    let (input_path, temp_input) = prepare_marp_input(project, slide)?;
    let make_command = |output_stem: String| {
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
            .arg(&input_path);

        cmd
    };

    let len = output_paths.len();

    Ok(output_paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| BuildCommand::HTML {
            dir: slide.dir.clone(),
            command: make_command(path),
            conf: slide.conf.clone(),
            temp_input: cleanup_temp_input(index, len, &temp_input),
        })
        .collect())
}

/// generate build commands for OGP images
pub fn build_ogp_image_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> anyhow::Result<Vec<BuildCommand>> {
    let output_paths = PublishPlan::for_slide(project, slide).ogp_image_paths;
    if output_paths.is_empty() {
        return Ok(vec![]);
    }

    let (input_path, temp_input) = prepare_marp_input(project, slide)?;
    let make_command = |output_path: String| {
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd.arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            .arg("--html")
            .arg("true")
            .arg("--image")
            .arg("png")
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(output_path),
            )
            .arg("--allow-local-files")
            .arg("--title")
            .arg(&slide.conf.name)
            .arg("--author")
            .arg(&project.conf.author)
            .arg("--description")
            .arg(slide.conf.description.clone().unwrap_or_default())
            .arg(&input_path);

        cmd
    };

    let len = output_paths.len();

    Ok(output_paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| BuildCommand::OGPImage {
            dir: slide.dir.clone(),
            command: make_command(path),
            conf: slide.conf.clone(),
            temp_input: cleanup_temp_input(index, len, &temp_input),
        })
        .collect())
}

/// copy ipe slide pdf to output directory
pub fn copy_ipe_pdf(
    project: &Project,
    slide: &Slide,
    include_latest_alias: bool,
) -> anyhow::Result<()> {
    let source_pdf_path = slide.dir.join("slide.pdf");
    let plan = PublishPlan::for_slide(project, slide);

    for stem in plan.versioned_pdf_stems {
        let pdf_save_path = project
            .root_dir
            .join(&project.conf.output_dir)
            .join(stem + ".pdf");

        std::fs::copy(&source_pdf_path, &pdf_save_path)?;
    }

    if include_latest_alias {
        for file_name in plan.latest_pdf_aliases {
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
    for stem in PublishPlan::for_slide(project, slide).html_paths {
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

pub fn write_alias_redirects(
    project: &Project,
    slide: &Slide,
    archived_slides: &[Slide],
) -> anyhow::Result<()> {
    let plan = PublishPlan::for_slide(project, slide);
    if plan.strategy != PathStrategy::CanonicalWithRedirects || slide.conf.draft.unwrap_or(false) {
        return Ok(());
    }

    write_redirect(
        project,
        &format!("{}/pdf/index.html", plan.canonical_stem),
        &absolute_url(
            project,
            &format!("{}_v{}.pdf", plan.canonical_stem, slide.conf.version),
        ),
        &absolute_url(project, &format!("{}/ogp.png", plan.canonical_stem)),
        slide,
    )?;

    for version in archived_slides.iter().chain(std::iter::once(slide)) {
        if version.conf.draft.unwrap_or(false) {
            continue;
        }

        write_redirect(
            project,
            &format!(
                "{}/pdf/v{}/index.html",
                plan.canonical_stem, version.conf.version
            ),
            &absolute_url(
                project,
                &format!("{}_v{}.pdf", plan.canonical_stem, version.conf.version),
            ),
            &absolute_url(
                project,
                &format!("{}/v{}/ogp.png", plan.canonical_stem, version.conf.version),
            ),
            &version,
        )?;
    }

    for alias in &plan.alias_stems {
        write_redirect(
            project,
            &format!("{alias}/index.html"),
            &absolute_url(project, &format!("{}/", plan.canonical_stem)),
            &absolute_url(project, &format!("{}/ogp.png", plan.canonical_stem)),
            slide,
        )?;

        write_redirect(
            project,
            &format!("{alias}/pdf/index.html"),
            &absolute_url(
                project,
                &format!("{}_v{}.pdf", plan.canonical_stem, slide.conf.version),
            ),
            &absolute_url(project, &format!("{}/ogp.png", plan.canonical_stem)),
            slide,
        )?;

        for version in archived_slides.iter().chain(std::iter::once(slide)) {
            if version.conf.draft.unwrap_or(false) {
                continue;
            }

            write_redirect(
                project,
                &format!("{alias}/v{}/index.html", version.conf.version),
                &absolute_url(
                    project,
                    &format!("{}/v{}/", plan.canonical_stem, version.conf.version),
                ),
                &absolute_url(
                    project,
                    &format!("{}/v{}/ogp.png", plan.canonical_stem, version.conf.version),
                ),
                &version,
            )?;

            write_redirect(
                project,
                &format!("{alias}/pdf/v{}/index.html", version.conf.version),
                &absolute_url(
                    project,
                    &format!("{}_v{}.pdf", plan.canonical_stem, version.conf.version),
                ),
                &absolute_url(
                    project,
                    &format!("{}/v{}/ogp.png", plan.canonical_stem, version.conf.version),
                ),
                &version,
            )?;
        }
    }

    Ok(())
}

fn write_redirect(
    project: &Project,
    relative_path: &str,
    target_url: &str,
    og_image_url: &str,
    slide: &Slide,
) -> anyhow::Result<()> {
    let output_path = project
        .root_dir
        .join(&project.conf.output_dir)
        .join(relative_path);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(
        output_path,
        redirect_html(project, slide, target_url, og_image_url),
    )?;
    Ok(())
}

fn redirect_html(project: &Project, slide: &Slide, target_url: &str, og_image_url: &str) -> String {
    let title = html_escape(&slide.conf.name);
    let description = html_escape(&slide.conf.description.clone().unwrap_or_default());
    let target = html_escape(target_url);
    let og_image = html_escape(og_image_url);
    let js_target = js_string_escape(target_url);
    let site_name = html_escape(&project.conf.name);

    format!(
        r#"<!doctype html>
<html lang="ja">
  <head>
    <meta charset="utf-8" />
    <title>{title}</title>
    <link rel="canonical" href="{target}" />
    <meta http-equiv="refresh" content="0; url={target}" />
    <meta name="description" content="{description}" />
    <meta property="og:type" content="website" />
    <meta property="og:title" content="{title}" />
    <meta property="og:description" content="{description}" />
    <meta property="og:url" content="{target}" />
    <meta property="og:site_name" content="{site_name}" />
    <meta property="og:image" content="{og_image}" />
    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="{title}" />
    <meta name="twitter:description" content="{description}" />
    <meta name="twitter:image" content="{og_image}" />
    <script>location.replace("{js_target}");</script>
  </head>
  <body>
    <p>Redirecting to <a href="{target}">{target}</a></p>
  </body>
</html>
"#
    )
}

fn absolute_url(project: &Project, path: &str) -> String {
    format!("{}/{}", project.conf.base_url.trim_end_matches('/'), path)
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn js_string_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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

#[cfg(test)]
mod test_build {
    use super::{
        build_ogp_image_commands, prepare_marp_input, write_alias_redirects, BuildCommand,
    };
    use crate::config::{BuildConf, PathStrategy, ProjectConf, SlideConf, SlideType, TemplateConf};
    use crate::project::Project;
    use crate::slide::Slide;

    #[test]
    fn prepare_marp_input_returns_original_when_suffix_is_empty() {
        let root = tempfile::tempdir().unwrap();
        let slide_dir = root.path().join("src").join("intro");
        std::fs::create_dir_all(&slide_dir).unwrap();
        std::fs::write(slide_dir.join("slide.md"), "# title\n").unwrap();

        let project = Project {
            root_dir: root.path().to_path_buf(),
            conf: ProjectConf {
                name: "demo".to_string(),
                author: "author".to_string(),
                base_url: "https://example.com".to_string(),
                output_dir: "output".to_string(),
                template: TemplateConf {
                    slide: String::new(),
                    index: String::new(),
                    suffix: String::new(),
                },
                build: BuildConf::default(),
            },
            slides: vec![],
        };
        let slide = Slide {
            dir: slide_dir.clone(),
            conf: SlideConf {
                name: "intro".to_string(),
                version: 1,
                secret: None,
                custom_path: None,
                draft: None,
                description: None,
                title_prefix: None,
                type_: SlideType::Marp,
                bibliography: None,
                path_strategy: None,
            },
        };

        let (input_path, temp_input) = prepare_marp_input(&project, &slide).unwrap();

        assert_eq!(input_path, slide_dir.join("slide.md"));
        assert_eq!(temp_input, None);
    }

    #[test]
    fn prepare_marp_input_appends_suffix_to_temp_file() {
        let root = tempfile::tempdir().unwrap();
        let slide_dir = root.path().join("src").join("intro");
        std::fs::create_dir_all(&slide_dir).unwrap();
        std::fs::write(slide_dir.join("slide.md"), "# title").unwrap();

        let project = Project {
            root_dir: root.path().to_path_buf(),
            conf: ProjectConf {
                name: "demo".to_string(),
                author: "author".to_string(),
                base_url: "https://example.com".to_string(),
                output_dir: "output".to_string(),
                template: TemplateConf {
                    slide: String::new(),
                    index: String::new(),
                    suffix: "<script src=\"/shared.js\"></script>".to_string(),
                },
                build: BuildConf::default(),
            },
            slides: vec![],
        };
        let slide = Slide {
            dir: slide_dir.clone(),
            conf: SlideConf {
                name: "intro".to_string(),
                version: 1,
                secret: None,
                custom_path: None,
                draft: None,
                description: None,
                title_prefix: None,
                type_: SlideType::Marp,
                bibliography: None,
                path_strategy: None,
            },
        };

        let (input_path, temp_input) = prepare_marp_input(&project, &slide).unwrap();
        let temp_input = temp_input.unwrap();
        let contents = std::fs::read_to_string(&input_path).unwrap();

        assert_eq!(input_path, temp_input);
        assert!(contents.ends_with("<script src=\"/shared.js\"></script>\n"));
        assert!(contents.starts_with("# title\n"));
    }

    #[test]
    fn write_alias_redirects_creates_html_and_pdf_redirects() {
        let root = tempfile::tempdir().unwrap();
        let output_dir = root.path().join("output");
        std::fs::create_dir_all(&output_dir).unwrap();

        let project = Project {
            root_dir: root.path().to_path_buf(),
            conf: ProjectConf {
                name: "demo".to_string(),
                author: "author".to_string(),
                base_url: "https://example.com/slides/".to_string(),
                output_dir: "output".to_string(),
                template: TemplateConf::default(),
                build: BuildConf {
                    theme_dir: ".marp/themes".to_string(),
                    marp_binary: "marp".to_string(),
                    path_strategy: PathStrategy::CanonicalWithRedirects,
                },
            },
            slides: vec![],
        };
        let latest = Slide {
            dir: root.path().join("src/intro"),
            conf: SlideConf {
                name: "intro".to_string(),
                version: 2,
                secret: Some("uuid".to_string()),
                custom_path: Some(vec!["talks".to_string()]),
                draft: None,
                description: Some("description".to_string()),
                title_prefix: None,
                type_: SlideType::Marp,
                bibliography: None,
                path_strategy: None,
            },
        };
        let archived = Slide {
            dir: root.path().join("src/intro/v1"),
            conf: SlideConf {
                version: 1,
                ..latest.conf.clone()
            },
        };

        write_alias_redirects(&project, &latest, &[archived]).unwrap();

        let latest_html = std::fs::read_to_string(output_dir.join("talks/index.html")).unwrap();
        let v1_html = std::fs::read_to_string(output_dir.join("talks/v1/index.html")).unwrap();
        let pdf_html = std::fs::read_to_string(output_dir.join("talks/pdf/index.html")).unwrap();
        let pdf_v2_html =
            std::fs::read_to_string(output_dir.join("talks/pdf/v2/index.html")).unwrap();
        let canonical_pdf_html =
            std::fs::read_to_string(output_dir.join("uuid/pdf/index.html")).unwrap();
        let canonical_pdf_v1_html =
            std::fs::read_to_string(output_dir.join("uuid/pdf/v1/index.html")).unwrap();
        let canonical_pdf_v2_html =
            std::fs::read_to_string(output_dir.join("uuid/pdf/v2/index.html")).unwrap();

        assert!(latest_html.contains("https://example.com/slides/uuid/"));
        assert!(latest_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/ogp.png" />"#
        ));
        assert!(latest_html.contains(
            r#"<meta name="twitter:image" content="https://example.com/slides/uuid/ogp.png" />"#
        ));
        assert!(v1_html.contains("https://example.com/slides/uuid/v1/"));
        assert!(v1_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/v1/ogp.png" />"#
        ));
        assert!(pdf_html.contains("https://example.com/slides/uuid_v2.pdf"));
        assert!(pdf_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/ogp.png" />"#
        ));
        assert!(pdf_v2_html.contains("https://example.com/slides/uuid_v2.pdf"));
        assert!(pdf_v2_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/v2/ogp.png" />"#
        ));
        assert!(canonical_pdf_html.contains("https://example.com/slides/uuid_v2.pdf"));
        assert!(canonical_pdf_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/ogp.png" />"#
        ));
        assert!(canonical_pdf_v1_html.contains("https://example.com/slides/uuid_v1.pdf"));
        assert!(canonical_pdf_v1_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/v1/ogp.png" />"#
        ));
        assert!(canonical_pdf_v2_html.contains("https://example.com/slides/uuid_v2.pdf"));
        assert!(canonical_pdf_v2_html.contains(
            r#"<meta property="og:image" content="https://example.com/slides/uuid/v2/ogp.png" />"#
        ));
    }

    #[test]
    fn build_ogp_image_commands_only_for_canonical_strategy() {
        let root = tempfile::tempdir().unwrap();
        let slide_dir = root.path().join("src").join("intro");
        std::fs::create_dir_all(&slide_dir).unwrap();
        std::fs::write(slide_dir.join("slide.md"), "# title\n").unwrap();

        let mut project = Project {
            root_dir: root.path().to_path_buf(),
            conf: ProjectConf {
                name: "demo".to_string(),
                author: "author".to_string(),
                base_url: "https://example.com".to_string(),
                output_dir: "output".to_string(),
                template: TemplateConf::default(),
                build: BuildConf::default(),
            },
            slides: vec![],
        };
        let slide = Slide {
            dir: slide_dir,
            conf: SlideConf {
                name: "intro".to_string(),
                version: 2,
                secret: Some("uuid".to_string()),
                custom_path: Some(vec!["talks".to_string()]),
                draft: None,
                description: None,
                title_prefix: None,
                type_: SlideType::Marp,
                bibliography: None,
                path_strategy: None,
            },
        };

        assert_eq!(build_ogp_image_commands(&project, &slide).unwrap().len(), 0);

        project.conf.build.path_strategy = PathStrategy::CanonicalWithRedirects;
        let commands = build_ogp_image_commands(&project, &slide).unwrap();

        assert_eq!(commands.len(), 2);
        assert!(commands
            .into_iter()
            .all(|command| matches!(command, BuildCommand::OGPImage { .. })));
    }
}
