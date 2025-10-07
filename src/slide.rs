use std::path::PathBuf;

use crate::{config::SlideConf, contents::SlideContents};

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

    /// get contents of slide
    pub fn get_contents(&self) -> anyhow::Result<SlideContents> {
        // slide string
        let slide_str = std::fs::read_to_string(self.slide_path())?;

        SlideContents::try_from(slide_str.as_str())
    }
}
