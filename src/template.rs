//! Template module

use askama::Template;

use crate::config::{ProjectConf, SlideConf};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub slides: &'a [SlideConf],
}

#[derive(Template)]
#[template(path = "readme.md")]
pub struct ReadmeTemplate<'a> {
    pub project: &'a ProjectConf,
    pub slides: &'a [SlideConf],
}

#[cfg(test)]
mod test_template {
    use askama::Template;

    use super::*;
    use crate::config::{BuildConf, SlideConf, TemplateConf};

    #[test]
    fn test_index() {
        let slides = vec![
            SlideConf {
                name: "title1".to_string(),
                version: 1,
                secret: None,
                custom_path: None,
                draft: None,
                description: None,
                title_prefix: None,
                bibliography: None,
            },
            SlideConf {
                name: "title2".to_string(),
                version: 1,
                secret: Some("uuid".to_string()),
                custom_path: None,
                draft: None,
                description: None,
                title_prefix: Some("#".to_string()),
                bibliography: None,
            },
            SlideConf {
                name: "title3".to_string(),
                version: 1,
                secret: None,
                custom_path: Some(vec!["path".to_string()]),
                draft: Some(true),
                description: None,
                title_prefix: Some("##".to_string()),
                bibliography: None,
            },
            SlideConf {
                name: "title4".to_string(),
                version: 1,
                secret: None,
                custom_path: None,
                draft: Some(false),
                description: Some("タイトル4".to_string()),
                title_prefix: Some("###".to_string()),
                bibliography: None,
            },
        ];
        let template = IndexTemplate { slides: &slides };

        eprintln!("{}", template.render().unwrap());
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
        };

        let project = ProjectConf {
            name: "my-project".to_string(),
            author: "powell".to_string(),
            base_url: "https://test.dev/slides/".to_string(),
            output_dir: "output".to_string(),
            template,
            build: build_conf,
        };

        let slides = vec![
            SlideConf {
                name: "title1".to_string(),
                version: 1,
                secret: None,
                custom_path: None,
                draft: None,
                description: None,
                title_prefix: None,
                bibliography: None,
            },
            SlideConf {
                name: "title2".to_string(),
                version: 1,
                secret: Some("uuid".to_string()),
                custom_path: None,
                draft: None,
                description: None,
                title_prefix: Some("#".to_string()),
                bibliography: None,
            },
            SlideConf {
                name: "title3".to_string(),
                version: 1,
                secret: None,
                custom_path: Some(vec!["path".to_string()]),
                draft: Some(true),
                description: None,
                title_prefix: Some("##".to_string()),
                bibliography: None,
            },
            SlideConf {
                name: "title4".to_string(),
                version: 1,
                secret: None,
                custom_path: None,
                draft: Some(false),
                description: Some("タイトル4".to_string()),
                title_prefix: Some("###".to_string()),
                bibliography: None,
            },
        ];

        let template = ReadmeTemplate {
            project: &project,
            slides: &slides,
        };

        eprintln!("{}", template.render().unwrap());
    }
}
