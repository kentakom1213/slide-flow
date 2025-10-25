use std::path::PathBuf;

use crate::{config::SlideConf, contents::SlideContents};

/// project information
#[derive(Debug, Clone)]
pub struct Slide {
    /// directory stores `slide.toml`
    pub dir: PathBuf,
    /// slide configuration
    pub conf: SlideConf,
    /// slide type (marp / ipe)
    pub type_: SlideType,
}

#[derive(Clone, Debug)]
pub enum SlideType {
    Marp,
    Ipe,
}

impl Slide {
    /// get path to slide file
    pub fn slide_path(&self) -> PathBuf {
        match self.type_ {
            SlideType::Marp => self.dir.join("slide.md"),
            SlideType::Ipe => self.dir.join("slide.ipe"),
        }
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
