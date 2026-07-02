//! Template module

use askama::Template;

use crate::{
    config::{PathStrategy, ProjectConf},
    path::PublishPlan,
    project::Project,
    slide::Slide,
};

#[derive(Debug, Clone)]
pub struct PublishedSlide {
    pub name: String,
    pub description: String,
    pub draft: bool,
    pub is_marp: bool,
    pub public: bool,
    pub slide_path: String,
    pub slide_version_paths: Vec<String>,
    pub pdf_path: String,
    pub pdf_version_paths: Vec<String>,
}

impl PublishedSlide {
    pub fn from_slide(project: &Project, slide: &Slide) -> Self {
        let plan = PublishPlan::for_slide(project, slide);
        let primary_stem = match plan.strategy {
            PathStrategy::Legacy => plan.canonical_stem.clone(),
            PathStrategy::CanonicalWithRedirects => plan
                .alias_stems
                .first()
                .cloned()
                .unwrap_or_else(|| plan.canonical_stem.clone()),
        };
        let public = match plan.strategy {
            PathStrategy::Legacy => slide.conf.secret.is_none(),
            PathStrategy::CanonicalWithRedirects => {
                slide.conf.secret.is_none() || !plan.alias_stems.is_empty()
            }
        };
        let slide_path = match plan.strategy {
            PathStrategy::Legacy => primary_stem.clone(),
            PathStrategy::CanonicalWithRedirects => format!("{primary_stem}/"),
        };
        let slide_version_paths = match plan.strategy {
            PathStrategy::Legacy => vec![],
            PathStrategy::CanonicalWithRedirects => (1..=slide.conf.version)
                .map(|version| format!("{primary_stem}/v{version}/"))
                .collect(),
        };
        let pdf_path = match plan.strategy {
            PathStrategy::Legacy => format!("{primary_stem}.pdf"),
            PathStrategy::CanonicalWithRedirects => format!("{primary_stem}/pdf/"),
        };
        let pdf_version_paths = (1..=slide.conf.version)
            .map(|version| match plan.strategy {
                PathStrategy::Legacy => format!("{primary_stem}_v{version}.pdf"),
                PathStrategy::CanonicalWithRedirects => format!("{primary_stem}/pdf/v{version}/"),
            })
            .collect();

        Self {
            name: slide.conf.name.clone(),
            description: slide.conf.description.clone().unwrap_or_default(),
            draft: slide.conf.draft.unwrap_or(false),
            is_marp: slide.conf.type_.is_marp(),
            public,
            slide_path,
            slide_version_paths,
            pdf_path,
            pdf_version_paths,
        }
    }
}

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub slides: &'a [PublishedSlide],
}

#[derive(Template)]
#[template(path = "readme.md")]
pub struct ReadmeTemplate<'a> {
    pub project: &'a ProjectConf,
    pub slides: &'a [PublishedSlide],
}

#[cfg(test)]
mod test_template {
    use askama::Template;

    use super::*;
    use crate::config::{BuildConf, TemplateConf};

    #[test]
    fn test_index() {
        let slides = vec![
            PublishedSlide {
                name: "title1".to_string(),
                description: String::new(),
                draft: false,
                is_marp: true,
                public: true,
                slide_path: "title1".to_string(),
                slide_version_paths: vec!["title1/v1/".to_string()],
                pdf_path: "title1.pdf".to_string(),
                pdf_version_paths: vec!["title1_v1.pdf".to_string()],
            },
            PublishedSlide {
                name: "title2".to_string(),
                description: String::new(),
                draft: false,
                is_marp: false,
                public: false,
                slide_path: "uuid".to_string(),
                slide_version_paths: vec![],
                pdf_path: "uuid.pdf".to_string(),
                pdf_version_paths: vec!["uuid_v1.pdf".to_string()],
            },
            PublishedSlide {
                name: "title3".to_string(),
                description: String::new(),
                draft: true,
                is_marp: true,
                public: true,
                slide_path: "path".to_string(),
                slide_version_paths: vec![],
                pdf_path: "path.pdf".to_string(),
                pdf_version_paths: vec!["path_v1.pdf".to_string()],
            },
            PublishedSlide {
                name: "title4".to_string(),
                description: "タイトル4".to_string(),
                draft: false,
                is_marp: true,
                public: true,
                slide_path: "title4".to_string(),
                slide_version_paths: vec![],
                pdf_path: "title4.pdf".to_string(),
                pdf_version_paths: vec!["title4_v1.pdf".to_string()],
            },
        ];
        let template = IndexTemplate { slides: &slides };

        let result = template.render().expect("Failed to format");
        eprintln!("{result}");
    }

    #[test]
    fn test_readme() {
        let template = TemplateConf {
            slide: String::new(),
            index: String::new(),
            suffix: String::new(),
        };

        let build_conf = BuildConf {
            theme_dir: String::new(),
            marp_binary: String::new(),
            path_strategy: Default::default(),
        };

        let project = ProjectConf {
            name: "my-project".to_string(),
            author: "powell".to_string(),
            base_url: "https://test.dev/slides/".to_string(),
            output_dir: "output".to_string(),
            template,
            build: build_conf,
            images: Default::default(),
        };

        let slides = vec![
            PublishedSlide {
                name: "title1".to_string(),
                description: String::new(),
                draft: false,
                is_marp: true,
                public: true,
                slide_path: "title1".to_string(),
                slide_version_paths: vec!["title1/v1/".to_string()],
                pdf_path: "title1.pdf".to_string(),
                pdf_version_paths: vec!["title1_v1.pdf".to_string()],
            },
            PublishedSlide {
                name: "title2".to_string(),
                description: String::new(),
                draft: false,
                is_marp: false,
                public: false,
                slide_path: "uuid".to_string(),
                slide_version_paths: vec![],
                pdf_path: "uuid.pdf".to_string(),
                pdf_version_paths: vec!["uuid_v1.pdf".to_string()],
            },
            PublishedSlide {
                name: "title3".to_string(),
                description: String::new(),
                draft: true,
                is_marp: true,
                public: true,
                slide_path: "path".to_string(),
                slide_version_paths: vec![],
                pdf_path: "path.pdf".to_string(),
                pdf_version_paths: vec!["path_v1.pdf".to_string()],
            },
            PublishedSlide {
                name: "title4".to_string(),
                description: "タイトル4".to_string(),
                draft: false,
                is_marp: true,
                public: true,
                slide_path: "title4".to_string(),
                slide_version_paths: vec![],
                pdf_path: "title4.pdf".to_string(),
                pdf_version_paths: vec!["title4_v1.pdf".to_string()],
            },
        ];

        let template = ReadmeTemplate {
            project: &project,
            slides: &slides,
        };

        let result = template.render().expect("Failed to format");
        assert!(result.contains("# slides

[Slides List](https://test.dev/slides/)

| Title | Slide | PDF | Description |
| :---- | :---: | :-: | :---------- |
| title1 | [Slide](https://test.dev/slides/title1),[v1](https://test.dev/slides/title1/v1/) | [PDF](https://test.dev/slides/title1.pdf),[v1](https://test.dev/slides/title1_v1.pdf) |  |
| title2 |  -  | [PDF](https://test.dev/slides/uuid.pdf),[v1](https://test.dev/slides/uuid_v1.pdf) |  |
| title3 | - | - |  |
| title4 | [Slide](https://test.dev/slides/title4) | [PDF](https://test.dev/slides/title4.pdf),[v1](https://test.dev/slides/title4_v1.pdf) | タイトル4 |"));
    }
}
