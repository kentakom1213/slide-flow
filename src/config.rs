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
    /// bibliography entries
    pub bibliography: Option<Vec<BibEntry>>,
}

/// bibliography entry
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BibEntry {
    /// citation tag
    pub tag: String,
    /// title of the reference
    pub title: String,
    /// authors of the reference
    pub authors: Option<String>,
    /// year of the reference
    pub year: u16,
    /// conference or journal name
    pub venue: Option<String>,
    /// URL
    pub url: Option<String>,
}

impl BibEntry {
    /// format bibliography entry as a string
    pub fn format(&self) -> String {
        let mut entry = String::new();

        if let Some(authors) = &self.authors {
            entry.push_str(authors);
            entry.push_str(". ");
        }

        entry.push_str(&self.title);
        entry.push_str(". ");

        if let Some(venue) = &self.venue {
            entry.push_str(venue);
            entry.push_str(", ");
        }

        entry.push_str(&self.year.to_string());

        if let Some(url) = &self.url {
            entry.push_str(". ");
            entry.push_str(url);
        }

        entry
    }
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

    #[test]
    fn test_parse_slide_config_bibliography() {
        let config_example = r###"
            version = 1
            name = "slide1"
            path = "slide1"
            draft = true
            description = "This is slide1"
            title_prefix = "##"

            [[bibliography]]
            tag = "tag1"
            authors = "Author A, Author B"
            title = "This is bibliographic information 1"
            year = 2021
            venue = "Conference X"
            url = "https://doi.org/xxxx"

            [[bibliography]]
            tag = "tag2"
            authors = "Author C"
            title = "This is bibliographic information 2"
            year = 2020
            url = "https://doi.org/yyyy"
        "###;

        let config: SlideConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);

        assert_eq!(config.bibliography.as_ref().unwrap().len(), 2);
        assert_eq!(
            config.bibliography.as_ref().unwrap()[0].format(),
            "Author A, Author B. This is bibliographic information 1. Conference X, 2021. https://doi.org/xxxx"
        );
        assert_eq!(
            config.bibliography.as_ref().unwrap()[1].format(),
            "Author C. This is bibliographic information 2. 2020. https://doi.org/yyyy"
        );
    }
}
