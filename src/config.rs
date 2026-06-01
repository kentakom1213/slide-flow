//! configration

use clap::ValueEnum;
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
    /// image optimization configuration
    #[serde(default)]
    pub images: ImagesConf,
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
            images: ImagesConf::default(),
        }
    }
}

/// image optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagesConf {
    /// enable image optimization
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// cache directory relative to the project root
    #[serde(default = "default_image_cache_dir")]
    pub cache_dir: String,
    /// optimization mode
    #[serde(default)]
    pub mode: ImageOptimizeMode,
    /// strip metadata when supported by the optimizer
    #[serde(default = "default_true")]
    pub strip_metadata: bool,
    /// fail instead of copying through when an optimizer is missing
    #[serde(default)]
    pub fail_on_missing_tool: bool,
    /// PNG optimizer configuration
    #[serde(default)]
    pub png: PngImageConf,
    /// JPEG optimizer configuration
    #[serde(default)]
    pub jpeg: JpegImageConf,
    /// SVG optimizer configuration
    #[serde(default)]
    pub svg: SvgImageConf,
    /// WebP handling configuration
    #[serde(default)]
    pub webp: WebpImageConf,
}

impl Default for ImagesConf {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: default_image_cache_dir(),
            mode: ImageOptimizeMode::Lossless,
            strip_metadata: true,
            fail_on_missing_tool: false,
            png: PngImageConf::default(),
            jpeg: JpegImageConf::default(),
            svg: SvgImageConf::default(),
            webp: WebpImageConf::default(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageOptimizeMode {
    #[default]
    Lossless,
    Lossy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PngImageConf {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_oxipng")]
    pub tool: String,
    #[serde(default = "default_png_level")]
    pub level: u8,
}

impl Default for PngImageConf {
    fn default() -> Self {
        Self {
            enabled: true,
            tool: default_oxipng(),
            level: default_png_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JpegImageConf {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_jpegoptim")]
    pub tool: String,
    #[serde(default = "default_jpeg_quality")]
    pub quality: u8,
}

impl Default for JpegImageConf {
    fn default() -> Self {
        Self {
            enabled: true,
            tool: default_jpegoptim(),
            quality: default_jpeg_quality(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvgImageConf {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_svgo")]
    pub tool: String,
}

impl Default for SvgImageConf {
    fn default() -> Self {
        Self {
            enabled: true,
            tool: default_svgo(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebpImageConf {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for WebpImageConf {
    fn default() -> Self {
        Self { enabled: false }
    }
}

fn default_true() -> bool {
    true
}

fn default_image_cache_dir() -> String {
    ".slide-flow/cache/images".to_string()
}

fn default_oxipng() -> String {
    "oxipng".to_string()
}

fn default_jpegoptim() -> String {
    "jpegoptim".to_string()
}

fn default_svgo() -> String {
    "svgo".to_string()
}

fn default_png_level() -> u8 {
    4
}

fn default_jpeg_quality() -> u8 {
    85
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
    /// default path strategy
    #[serde(default)]
    pub path_strategy: PathStrategy,
}

impl Default for BuildConf {
    /// Provides default build configurations.
    fn default() -> Self {
        BuildConf {
            theme_dir: ".marp/themes".to_string(),
            marp_binary: "marp".to_string(),
            path_strategy: PathStrategy::Legacy,
        }
    }
}

/// strategy for published paths
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathStrategy {
    Legacy,
    CanonicalWithRedirects,
}

impl Default for PathStrategy {
    fn default() -> Self {
        Self::Legacy
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
    /// type of slide.
    #[serde(rename = "type", default)]
    pub type_: SlideType,
    /// bibliography entries
    pub bibliography: Option<Vec<BibEntry>>,
    /// slide-local path strategy override
    pub path_strategy: Option<PathStrategy>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, ValueEnum)]
pub enum SlideType {
    #[default]
    Marp,
    Ipe,
}

impl SlideType {
    pub fn is_marp(&self) -> bool {
        matches!(self, Self::Marp)
    }

    pub fn is_ipe(&self) -> bool {
        matches!(self, Self::Ipe)
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            Self::Marp => "slide.md",
            Self::Ipe => "slide.ipe",
        }
    }
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
            path_strategy = "legacy"
        "##;

        let config: ProjectConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);
        assert_eq!(config.build.path_strategy, PathStrategy::Legacy);
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
            path_strategy = "canonical-with-redirects"
        "###;

        let config: SlideConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);
        assert_eq!(
            config.path_strategy,
            Some(PathStrategy::CanonicalWithRedirects)
        );
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
            path_strategy = "legacy"

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
        assert_eq!(config.path_strategy, Some(PathStrategy::Legacy));
    }

    #[test]
    fn test_parse_legacy_configs_without_path_strategy() {
        let project_config = r##"
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

        let slide_config = r###"
            version = 1
            name = "slide1"
        "###;

        let project: ProjectConf = toml::from_str(project_config).unwrap();
        let slide: SlideConf = toml::from_str(slide_config).unwrap();

        assert_eq!(project.build.path_strategy, PathStrategy::Legacy);
        assert_eq!(slide.path_strategy, None);
    }
}
