use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::bail;

use crate::{
    config::PathStrategy,
    path::{alias_stems, canonical_stem, PublishPlan},
    project::Project,
    slide::Slide,
    subcommand::build::{
        build, build_html_commands, build_ogp_image_commands, build_pdf_commands,
        build_pdf_latest_alias_commands, copy_images_html, copy_ipe_pdf, write_alias_redirects,
    },
};

pub fn plan(project: &Project, dir: Option<PathBuf>) -> anyhow::Result<()> {
    for slide in target_slides(project, dir.as_deref())? {
        let publish = PublishPlan::for_slide(project, &slide);
        println!("slide: {}", relative_path(project, &slide.dir));
        println!("  current_strategy: {:?}", publish.strategy);
        println!("  target_strategy: CanonicalWithRedirects");
        println!("  metadata: set path_strategy = \"canonical-with-redirects\"");
        println!("  canonical: {}", publish.canonical_stem);
        println!("  aliases: {}", display_list(&alias_stems(&slide)));
        println!("  redirects: {}", display_redirects(project, &slide)?);
        println!();
    }

    Ok(())
}

pub fn status(project: &Project) -> anyhow::Result<()> {
    for slide in &project.slides {
        let publish = PublishPlan::for_slide(project, slide);
        println!("slide: {}", relative_path(project, &slide.dir));
        println!("  strategy: {:?}", publish.strategy);
        println!("  canonical: {}", publish.canonical_stem);
        println!("  aliases: {}", display_list(&publish.alias_stems));
        println!(
            "  canonical_artifacts: {}",
            if canonical_artifacts_exist(project, slide)? {
                "ok"
            } else {
                "missing"
            }
        );
        println!(
            "  redirects: {}",
            if redirects_exist(project, slide)? {
                "ok"
            } else {
                "missing"
            }
        );
        println!();
    }

    Ok(())
}

pub struct ApplyOptions {
    pub metadata_only: bool,
    pub redirects_only: bool,
    pub artifacts: bool,
    pub remove_legacy_artifacts: bool,
    pub concurrent: usize,
}

pub fn apply(project: &Project, dir: PathBuf, options: ApplyOptions) -> anyhow::Result<()> {
    let mode_count = [
        options.metadata_only,
        options.redirects_only,
        options.artifacts,
        options.remove_legacy_artifacts,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();

    if mode_count == 0 {
        bail!("Please specify one of --metadata-only, --redirects-only, --artifacts, or --remove-legacy-artifacts");
    }
    if mode_count > 1 {
        bail!("Please specify only one migration mode at a time");
    }

    let slide = project.get_slide_root(&dir)?;

    if options.metadata_only {
        set_slide_path_strategy(&slide, PathStrategy::CanonicalWithRedirects)?;
        println!(
            "updated: {}",
            slide.dir.join("slide.toml").to_string_lossy()
        );
        return Ok(());
    }

    if options.redirects_only {
        write_redirects(project, &slide)?;
        return Ok(());
    }

    if options.artifacts {
        set_slide_path_strategy(&slide, PathStrategy::CanonicalWithRedirects)?;
        let slide = project.get_slide_root(&dir)?;
        build_artifacts(project, &slide, options.concurrent)?;
        write_redirects(project, &slide)?;
        return Ok(());
    }

    if options.remove_legacy_artifacts {
        remove_legacy_artifacts(project, &slide)?;
        return Ok(());
    }

    Ok(())
}

fn target_slides(project: &Project, dir: Option<&Path>) -> anyhow::Result<Vec<Slide>> {
    if let Some(dir) = dir {
        return Ok(vec![project.get_slide_root(dir)?]);
    }

    Ok(project.slides.clone())
}

fn set_slide_path_strategy(slide: &Slide, strategy: PathStrategy) -> anyhow::Result<()> {
    let conf_path = slide.dir.join("slide.toml");
    let conf_str = fs::read_to_string(&conf_path)?;
    let mut conf = slide.conf.clone();
    conf.path_strategy = Some(strategy);
    let new_conf_str = toml::to_string(&conf)?;

    if conf_str != new_conf_str {
        fs::write(conf_path, new_conf_str)?;
    }

    Ok(())
}

fn write_redirects(project: &Project, slide: &Slide) -> anyhow::Result<()> {
    let mut slide = slide.clone();
    slide.conf.path_strategy = Some(PathStrategy::CanonicalWithRedirects);
    let archived = project.get_archived_slides(&slide)?;
    write_alias_redirects(project, &slide, &archived)?;
    println!("redirects: {}", relative_path(project, &slide.dir));
    Ok(())
}

fn build_artifacts(project: &Project, slide: &Slide, concurrent: usize) -> anyhow::Result<()> {
    let archived_slides = project.get_archived_slides(slide)?;
    let mut cmds = vec![];

    if slide.conf.type_.is_ipe() {
        copy_ipe_pdf(project, slide, true)?;
        for archived in archived_slides {
            copy_ipe_pdf(project, &archived, false)?;
        }
        return Ok(());
    }

    copy_images_html(project, slide)?;
    cmds.extend(build_html_commands(project, slide)?);
    cmds.extend(build_pdf_commands(project, slide)?);
    cmds.extend(build_pdf_latest_alias_commands(project, slide)?);
    cmds.extend(build_ogp_image_commands(project, slide)?);

    for archived in archived_slides {
        let mut archived = archived;
        archived.conf.path_strategy = Some(PathStrategy::CanonicalWithRedirects);
        if archived.conf.type_.is_marp() {
            copy_images_html(project, &archived)?;
            cmds.extend(build_html_commands(project, &archived)?);
            cmds.extend(build_pdf_commands(project, &archived)?);
            cmds.extend(build_ogp_image_commands(project, &archived)?);
        }
    }

    build(cmds.into_iter(), concurrent);
    Ok(())
}

fn remove_legacy_artifacts(project: &Project, slide: &Slide) -> anyhow::Result<()> {
    let output_dir = project.root_dir.join(&project.conf.output_dir);
    let aliases = alias_stems(slide);

    for alias in aliases {
        remove_path(output_dir.join(format!("{alias}.pdf")))?;
        remove_path(output_dir.join(format!("{alias}_v{}.pdf", slide.conf.version)))?;
        remove_path(output_dir.join(&alias).join("images"))?;

        for archived in project.get_archived_slides(slide)? {
            remove_path(output_dir.join(format!("{alias}_v{}.pdf", archived.conf.version)))?;
        }
    }

    println!(
        "removed legacy alias artifacts: {}",
        relative_path(project, &slide.dir)
    );
    Ok(())
}

fn remove_path(path: PathBuf) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }

    Ok(())
}

fn canonical_artifacts_exist(project: &Project, slide: &Slide) -> anyhow::Result<bool> {
    let output_dir = project.root_dir.join(&project.conf.output_dir);
    let canonical = canonical_stem(slide);
    Ok(output_dir.join(&canonical).join("index.html").exists()
        && output_dir
            .join(format!("{}_v{}.pdf", canonical, slide.conf.version))
            .exists())
}

fn redirects_exist(project: &Project, slide: &Slide) -> anyhow::Result<bool> {
    let output_dir = project.root_dir.join(&project.conf.output_dir);
    let aliases = alias_stems(slide);
    if aliases.is_empty() {
        return Ok(true);
    }

    Ok(aliases
        .iter()
        .all(|alias| output_dir.join(alias).join("index.html").exists()))
}

fn display_redirects(project: &Project, slide: &Slide) -> anyhow::Result<String> {
    let aliases = alias_stems(slide);
    if aliases.is_empty() {
        return Ok("-".to_string());
    }

    let mut redirects = vec![];
    for alias in aliases {
        redirects.push(format!("{alias}/ -> {}/", canonical_stem(slide)));
        redirects.push(format!(
            "{alias}/pdf/ -> {}_v{}.pdf",
            canonical_stem(slide),
            slide.conf.version
        ));
        for archived in project.get_archived_slides(slide)? {
            redirects.push(format!(
                "{alias}/v{}/ -> {}/v{}/",
                archived.conf.version,
                canonical_stem(slide),
                archived.conf.version
            ));
        }
    }

    Ok(redirects.join(", "))
}

fn display_list(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(", ")
    }
}

fn relative_path(project: &Project, path: &Path) -> String {
    path.strip_prefix(&project.root_dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        config::{PathStrategy, SlideType},
        project::Project,
        subcommand::{
            add::add,
            init::init,
            migrate::{apply, ApplyOptions},
        },
    };

    #[test]
    fn apply_metadata_only_sets_slide_strategy() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false, SlideType::Marp).unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        apply(
            &project,
            PathBuf::from("src/intro"),
            ApplyOptions {
                metadata_only: true,
                redirects_only: false,
                artifacts: false,
                remove_legacy_artifacts: false,
                concurrent: 4,
            },
        )
        .unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let slide = project.get_slide_root(Path::new("src/intro")).unwrap();

        assert_eq!(
            slide.conf.path_strategy,
            Some(PathStrategy::CanonicalWithRedirects)
        );
    }

    #[test]
    fn remove_legacy_artifacts_keeps_alias_redirect_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false, SlideType::Marp).unwrap();
        let slide_conf_path = root.join("src/intro/slide.toml");
        let slide_conf = std::fs::read_to_string(&slide_conf_path).unwrap();
        std::fs::write(
            &slide_conf_path,
            slide_conf.replace("custom_path = []", "custom_path = [\"talks\"]"),
        )
        .unwrap();
        std::fs::create_dir_all(root.join("output/talks/images")).unwrap();
        std::fs::write(root.join("output/talks/index.html"), "redirect").unwrap();
        std::fs::write(root.join("output/talks.pdf"), "pdf").unwrap();
        std::fs::write(root.join("output/talks_v1.pdf"), "pdf").unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        apply(
            &project,
            PathBuf::from("src/intro"),
            ApplyOptions {
                metadata_only: false,
                redirects_only: false,
                artifacts: false,
                remove_legacy_artifacts: true,
                concurrent: 4,
            },
        )
        .unwrap();

        assert!(root.join("output/talks/index.html").exists());
        assert!(!root.join("output/talks/images").exists());
        assert!(!root.join("output/talks.pdf").exists());
        assert!(!root.join("output/talks_v1.pdf").exists());
    }
}
