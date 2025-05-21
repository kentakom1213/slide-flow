//! configration

use serde::{Deserialize, Serialize};

/// configuration for project
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConf {
    /// name of the project
    pub name: String,
    /// author of the project
    pub author: String,
    /// Base URL for the project
    pub base_url: String,
    /// output directory
    pub output_dir: String,
    /// template configuration
    pub template: TemplateConf,
    /// build configuration
    pub build: BuildConf,
}

impl Default for ProjectConf {
    /// Provides a default configuration for a new project.
    fn default() -> Self {
        ProjectConf {
            name: "my-slide-project".to_string(), // Default project name
            author: "Your Name".to_string(),      // Default author name
            base_url: "https://example.com/".to_string(),
            output_dir: "output".to_string(),
            template: TemplateConf::default(),
            build: BuildConf::default(),
        }
    }
}

/// template configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateConf {
    /// template for slide
    pub slide: String,
    /// template for index
    pub index: String,
    /// suffix for slide
    pub suffix: String,
}

impl Default for TemplateConf {
    /// Provides default template configurations.
    fn default() -> Self {
        TemplateConf {
            slide: "".to_string(),
            index: "".to_string(),
            suffix: "".to_string(),
        }
    }
}

/// build configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct BuildConf {
    /// theme directory
    pub theme_dir: String,
    /// binary for marp
    pub marp_binary: String,
}

impl Default for BuildConf {
    /// Provides default build configurations.
    fn default() -> Self {
        BuildConf {
            theme_dir: ".marp/themes".to_string(),
            marp_binary: "marp".to_string(),
        }
    }
}

/// configuration for slide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideConf {
    /// name of the slide
    pub name: String,
    /// version of the slide
    pub version: u8,
    /// UUID (when secret slide)
    pub secret: Option<String>,
    /// custom path for slide
    pub custom_path: Option<Vec<String>>,
    /// draft flag
    /// - if true, the slide is not published
    pub draft: Option<bool>,
    /// description of the slide
    pub description: Option<String>,
    /// prefix of the title
    pub title_prefix: Option<String>,
}

#[cfg(test)]
mod test_config {
    use super::*;

    #[test]
    fn test_parse_project_config() {
        let config_example = r##"
            name = "slide-flow"
            author = "powell"
            base_url = "https://test.dev/"
            output_dir = "output"
            
            [template]
            slide = "<!-- slide -->"
            index = "<!-- index -->"
            suffix = "<!-- slide-end -->"

            [build]
            theme_dir = ".marp/themes"
            marp_binary = "marp"
        "##;

        let config: ProjectConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);
    }

    #[test]
    fn test_parse_slide_config() {
        let config_example = r###"
            version = 1
            name = "slide1"
            path = "slide1"
            draft = true
            description = "This is slide1"
            title_prefix = "##"
        "###;

        let config: SlideConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);
    }
}
