use std::path::Path;

use anyhow::bail;

use crate::{
    project::Project,
    slide::Slide,
    subcommand::build::{make_file_stems, make_latest_pdf_aliases, make_versioned_stems},
};

pub fn show(project: &Project, selector: &str) -> anyhow::Result<()> {
    let slide = resolve_selector(project, selector)?;
    println!("{}", render(project, &slide)?);
    Ok(())
}

fn resolve_selector(project: &Project, selector: &str) -> anyhow::Result<Slide> {
    if let Ok(number) = selector.parse::<usize>() {
        if number == 0 {
            bail!("Slide number must be 1 or greater");
        }
        return project.get_slide_by_index(number - 1);
    }

    project.get_slide_root(Path::new(selector))
}

fn render(project: &Project, slide: &Slide) -> anyhow::Result<String> {
    let archived = project.get_archived_slides(slide)?;
    let versions = archived
        .into_iter()
        .chain(std::iter::once(slide.clone()))
        .collect::<Vec<_>>();

    let relative_dir = slide
        .dir
        .strip_prefix(&project.root_dir)
        .unwrap_or(&slide.dir)
        .display()
        .to_string();

    let custom_path = slide
        .conf
        .custom_path
        .clone()
        .unwrap_or_default()
        .join(", ");

    let mut lines = vec![
        format!("no: {}", slide_number(project, slide)),
        format!("path: {relative_dir}"),
        format!("name: {}", slide.conf.name),
        format!("version: {}", slide.conf.version),
        format!(
            "type: {}",
            if slide.conf.type_.is_marp() {
                "marp"
            } else {
                "ipe"
            }
        ),
        format!("draft: {}", slide.conf.draft.unwrap_or(false)),
        format!(
            "secret: {}",
            slide.conf.secret.clone().unwrap_or_else(|| "-".to_string())
        ),
        format!(
            "custom_path: {}",
            if custom_path.is_empty() {
                "-"
            } else {
                &custom_path
            }
        ),
        format!(
            "description: {}",
            slide
                .conf
                .description
                .clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "-".to_string())
        ),
        "urls:".to_string(),
    ];

    for version in versions {
        lines.push(format!("  v{}:", version.conf.version));

        if version.dir == slide.dir {
            for url in make_file_stems(&version)
                .into_iter()
                .map(|stem| join_url(&project.conf.base_url, &stem))
            {
                lines.push(format!("    html: {url}"));
            }

            for url in make_latest_pdf_aliases(&version)
                .into_iter()
                .map(|file| join_url(&project.conf.base_url, &file))
            {
                lines.push(format!("    pdf_latest: {url}"));
            }
        }

        for url in make_versioned_stems(&version)
            .into_iter()
            .map(|stem| join_url(&project.conf.base_url, &format!("{stem}.pdf")))
        {
            lines.push(format!("    pdf: {url}"));
        }
    }

    Ok(lines.join("\n"))
}

fn slide_number(project: &Project, slide: &Slide) -> usize {
    project
        .slides
        .iter()
        .position(|candidate| candidate.dir == slide.dir)
        .map(|idx| idx + 1)
        .unwrap_or(0)
}

fn join_url(base_url: &str, path: &str) -> String {
    format!("{}/{}", base_url.trim_end_matches('/'), path)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::{
        config::SlideType,
        project::Project,
        subcommand::{add::add, init::init, version::bump},
    };

    use super::{render, resolve_selector};

    #[test]
    fn test_resolve_selector_accepts_slide_number() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "alpha".to_string(), false, false, SlideType::Marp).unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let slide = resolve_selector(&project, "1").unwrap();

        assert_eq!(slide.conf.name, "alpha");
    }

    #[test]
    fn test_render_includes_urls_for_each_version() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let config = root.join("config.toml");
        let config_str = std::fs::read_to_string(&config).unwrap();
        let config_str =
            config_str.replace("https://example.com/", "https://slides.example.com/base/");
        std::fs::write(&config, config_str).unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false, SlideType::Marp).unwrap();
        let slide_conf_path = root.join("src/intro/slide.toml");
        let slide_conf = std::fs::read_to_string(&slide_conf_path).unwrap();
        let slide_conf = slide_conf.replace("custom_path = []", "custom_path = [\"talks\"]");
        std::fs::write(&slide_conf_path, slide_conf).unwrap();

        bump(
            &Project::get(root.to_path_buf()).unwrap(),
            PathBuf::from("src/intro"),
        )
        .unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let output = render(
            &project,
            &project.get_slide_root(Path::new("src/intro")).unwrap(),
        )
        .unwrap();

        assert!(output.contains("no: 1"));
        assert!(output.contains("path: src/intro"));
        assert!(output.contains("  v1:"));
        assert!(output.contains("    pdf: https://slides.example.com/base/talks_v1.pdf"));
        assert!(output.contains("    pdf: https://slides.example.com/base/intro_v1.pdf"));
        assert!(output.contains("  v2:"));
        assert!(output.contains("    html: https://slides.example.com/base/talks"));
        assert!(output.contains("    html: https://slides.example.com/base/intro"));
        assert!(output.contains("    pdf_latest: https://slides.example.com/base/talks.pdf"));
        assert!(output.contains("    pdf_latest: https://slides.example.com/base/intro.pdf"));
        assert!(output.contains("    pdf: https://slides.example.com/base/talks_v2.pdf"));
        assert!(output.contains("    pdf: https://slides.example.com/base/intro_v2.pdf"));
    }
}
