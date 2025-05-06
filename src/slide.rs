use std::path::PathBuf;

use crate::config::SlideConf;

/// project information
#[derive(Debug, Clone)]
pub struct Slide {
    /// directory stores `slide.toml`
    pub dir: PathBuf,
    /// slide configuration
    pub conf: SlideConf,
}

impl Slide {
    /// get path to slide file
    pub fn slide_path(&self) -> PathBuf {
        self.dir.join("slide.md")
    }

    /// get path to images of the slide
    pub fn image_dir(&self) -> PathBuf {
        self.dir.join("images")
    }
}
