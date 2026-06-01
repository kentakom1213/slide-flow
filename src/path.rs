use crate::{config::PathStrategy, project::Project, slide::Slide};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishPlan {
    pub strategy: PathStrategy,
    pub canonical_stem: String,
    pub alias_stems: Vec<String>,
    pub html_stems: Vec<String>,
    pub html_paths: Vec<String>,
    pub ogp_image_paths: Vec<String>,
    pub versioned_pdf_stems: Vec<String>,
    pub latest_pdf_aliases: Vec<String>,
}

impl PublishPlan {
    pub fn for_slide(project: &Project, slide: &Slide) -> Self {
        let strategy = project.path_strategy(slide);

        match strategy {
            PathStrategy::Legacy => legacy_publish_plan(slide, strategy),
            PathStrategy::CanonicalWithRedirects => canonical_publish_plan(slide, strategy),
        }
    }
}

pub fn canonical_stem(slide: &Slide) -> String {
    slide
        .conf
        .secret
        .clone()
        .unwrap_or_else(|| slide.conf.name.clone())
}

pub fn alias_stems(slide: &Slide) -> Vec<String> {
    slide.conf.custom_path.clone().unwrap_or_default()
}

pub fn legacy_file_stems(slide: &Slide) -> Vec<String> {
    let mut res = alias_stems(slide);
    res.push(canonical_stem(slide));
    res
}

fn legacy_publish_plan(slide: &Slide, strategy: PathStrategy) -> PublishPlan {
    let html_stems = legacy_file_stems(slide);
    let versioned_pdf_stems = html_stems
        .iter()
        .map(|stem| format!("{stem}_v{}", slide.conf.version))
        .collect();
    let latest_pdf_aliases = html_stems
        .iter()
        .map(|stem| format!("{stem}.pdf"))
        .collect();

    PublishPlan {
        strategy,
        canonical_stem: canonical_stem(slide),
        alias_stems: alias_stems(slide),
        html_paths: html_stems.clone(),
        html_stems,
        ogp_image_paths: vec![],
        versioned_pdf_stems,
        latest_pdf_aliases,
    }
}

fn canonical_publish_plan(slide: &Slide, strategy: PathStrategy) -> PublishPlan {
    let canonical_stem = canonical_stem(slide);
    let version_path = format!("{canonical_stem}/v{}", slide.conf.version);
    let html_paths = if is_archived_slide(slide) {
        vec![version_path]
    } else {
        vec![canonical_stem.clone(), version_path]
    };
    let versioned_pdf_stems = vec![format!("{}_v{}", canonical_stem, slide.conf.version)];

    PublishPlan {
        strategy,
        canonical_stem: canonical_stem.clone(),
        alias_stems: alias_stems(slide),
        html_stems: vec![canonical_stem],
        ogp_image_paths: html_paths
            .iter()
            .map(|path| format!("{path}/ogp.png"))
            .collect(),
        html_paths,
        versioned_pdf_stems,
        latest_pdf_aliases: vec![],
    }
}

fn is_archived_slide(slide: &Slide) -> bool {
    slide
        .dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('v') && name[1..].chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{BuildConf, PathStrategy, ProjectConf, SlideConf, SlideType, TemplateConf},
        project::Project,
        slide::Slide,
    };

    use super::{alias_stems, canonical_stem, legacy_file_stems, PublishPlan};

    fn project(path_strategy: PathStrategy) -> Project {
        Project {
            root_dir: std::path::PathBuf::from("/tmp/project"),
            conf: ProjectConf {
                name: "demo".to_string(),
                author: "author".to_string(),
                base_url: "https://example.com".to_string(),
                output_dir: "output".to_string(),
                template: TemplateConf::default(),
                build: BuildConf {
                    theme_dir: ".marp/themes".to_string(),
                    marp_binary: "marp".to_string(),
                    path_strategy,
                },
            },
            slides: vec![],
        }
    }

    fn slide(path_strategy: Option<PathStrategy>) -> Slide {
        Slide {
            dir: std::path::PathBuf::from("/tmp/project/src/intro"),
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
                path_strategy,
            },
        }
    }

    #[test]
    fn splits_canonical_alias_and_legacy_stems() {
        let slide = slide(None);

        assert_eq!(canonical_stem(&slide), "uuid");
        assert_eq!(alias_stems(&slide), vec!["talks"]);
        assert_eq!(legacy_file_stems(&slide), vec!["talks", "uuid"]);
    }

    #[test]
    fn legacy_publish_plan_preserves_existing_outputs() {
        let project = project(PathStrategy::Legacy);
        let slide = slide(None);
        let plan = PublishPlan::for_slide(&project, &slide);

        assert_eq!(plan.strategy, PathStrategy::Legacy);
        assert_eq!(plan.html_stems, vec!["talks", "uuid"]);
        assert_eq!(plan.html_paths, vec!["talks", "uuid"]);
        assert_eq!(plan.ogp_image_paths, Vec::<String>::new());
        assert_eq!(plan.versioned_pdf_stems, vec!["talks_v2", "uuid_v2"]);
        assert_eq!(plan.latest_pdf_aliases, vec!["talks.pdf", "uuid.pdf"]);
    }

    #[test]
    fn slide_path_strategy_overrides_project_default() {
        let project = project(PathStrategy::Legacy);
        let slide = slide(Some(PathStrategy::CanonicalWithRedirects));
        let plan = PublishPlan::for_slide(&project, &slide);

        assert_eq!(plan.strategy, PathStrategy::CanonicalWithRedirects);
        assert_eq!(plan.canonical_stem, "uuid");
        assert_eq!(plan.alias_stems, vec!["talks"]);
        assert_eq!(plan.html_stems, vec!["uuid"]);
        assert_eq!(plan.html_paths, vec!["uuid", "uuid/v2"]);
        assert_eq!(
            plan.ogp_image_paths,
            vec!["uuid/ogp.png", "uuid/v2/ogp.png"]
        );
        assert_eq!(plan.versioned_pdf_stems, vec!["uuid_v2"]);
        assert_eq!(plan.latest_pdf_aliases, Vec::<String>::new());
    }

    #[test]
    fn archived_canonical_publish_plan_uses_versioned_html_only() {
        let project = project(PathStrategy::CanonicalWithRedirects);
        let mut slide = slide(None);
        slide.dir = std::path::PathBuf::from("/tmp/project/src/intro/v1");
        slide.conf.version = 1;

        let plan = PublishPlan::for_slide(&project, &slide);

        assert_eq!(plan.html_paths, vec!["uuid/v1"]);
        assert_eq!(plan.ogp_image_paths, vec!["uuid/v1/ogp.png"]);
        assert_eq!(plan.versioned_pdf_stems, vec!["uuid_v1"]);
    }
}
