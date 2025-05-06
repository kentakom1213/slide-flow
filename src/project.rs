use std::{fs, path::PathBuf};

use anyhow::bail;
use itertools::Itertools;

use crate::{
    config::{ProjectConf, SlideConf},
    slide::Slide,
};

/// project information
#[derive(Debug)]
pub struct Project {
    /// directory stores `config.toml`
    pub root_dir: PathBuf,
    /// project configuration
    pub conf: ProjectConf,
    /// list of slides
    pub slides: Vec<Slide>,
}

impl Project {
    /// create a new project
    pub fn get(root_dir: PathBuf) -> anyhow::Result<Self> {
        // read project configuration
        let conf_path = root_dir.join("config.toml");
        let Ok(conf_str) = std::fs::read_to_string(&conf_path) else {
            bail!("The project config file does not exist: {:?}", conf_path);
        };
        let conf: ProjectConf = toml::from_str(&conf_str)?;
        // get slide list
        let slides = Self::get_slide_list(&root_dir)?;

        Ok(Self {
            root_dir,
            conf,
            slides,
        })
    }

    /// get slide list
    fn get_slide_list(root_dir: &PathBuf) -> anyhow::Result<Vec<Slide>> {
        // get all directories in `src` directory
        let slides = fs::read_dir(root_dir.join("src"))?
            .filter_map(|e| e.ok())
            .filter_map(|dir| Self::get_slide_inner(&root_dir, &dir.path()).ok())
            // sort by directory name
            .sorted_by(|a, b| a.dir.cmp(&b.dir))
            .collect::<Vec<_>>();

        Ok(slides)
    }

    /// get slide list
    fn get_slide_inner(root_dir: &PathBuf, dir: &PathBuf) -> anyhow::Result<Slide> {
        // directory name
        let dir = root_dir.join(&dir);
        // path to config file
        let conf_path = dir.join("slide.toml");
        // read config file
        let Ok(conf_str) = std::fs::read_to_string(&conf_path) else {
            bail!("The project config file does not exist: {:?}", conf_path);
        };
        let conf: SlideConf = toml::from_str(&conf_str)?;

        Ok(Slide { dir, conf })
    }

    /// get config files for all slides
    pub fn get_slide_conf_list(&self) -> Vec<SlideConf> {
        self.slides.iter().map(|slide| slide.conf.clone()).collect()
    }

    /// get specific slide
    pub fn get_slide(&self, dir: &PathBuf) -> anyhow::Result<Slide> {
        Self::get_slide_inner(&self.root_dir, dir)
    }
}
