use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::bail;
use itertools::Itertools;

use crate::{
    config::{ProjectConf, SlideConf},
    slide::{Slide, SlideType},
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
            bail!(
                "The project config file does not exist: {}",
                conf_path.to_string_lossy()
            );
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
    fn get_slide_list(root_dir: &Path) -> anyhow::Result<Vec<Slide>> {
        // get all directories in `src` directory
        let slides = fs::read_dir(root_dir.join("src"))?
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().is_dir())
            .filter_map(|dir| Self::get_slide_inner(root_dir, &dir.path()).ok())
            // sort by directory name
            .sorted_by(|a, b| a.dir.cmp(&b.dir))
            .collect::<Vec<_>>();

        Ok(slides)
    }

    /// get slide list
    fn get_slide_inner(root_dir: &Path, dir: &Path) -> anyhow::Result<Slide> {
        // directory name
        let dir = if dir.is_absolute() {
            dir.to_path_buf()
        } else {
            root_dir.join(dir)
        };
        // path to config file
        let conf_path = dir.join("slide.toml");
        // read config file
        let Ok(conf_str) = std::fs::read_to_string(&conf_path) else {
            bail!(
                "The slide config file does not exist: {}",
                conf_path.to_string_lossy()
            );
        };
        let conf: SlideConf = toml::from_str(&conf_str)?;
        // detect slide type
        let type_ = if dir.join("slide.md").exists() {
            SlideType::Marp
        } else if dir.join("slide.ipe").exists() {
            SlideType::Ipe
        } else {
            bail!("The slide file does not exist.")
        };

        Ok(Slide { dir, conf, type_ })
    }

    /// get config files for all slides
    pub fn get_slide_conf_list(&self) -> Vec<SlideConf> {
        self.slides.iter().map(|slide| slide.conf.clone()).collect()
    }

    /// get specific slide
    pub fn get_slide(&self, dir: &Path) -> anyhow::Result<Slide> {
        if let Ok(slide) = Self::get_slide_inner(&self.root_dir, dir) {
            return Ok(slide);
        }

        // support versioned source alias like `src/<slide>_v2`
        if let Some((base, version)) = split_versioned_alias(dir) {
            let base_dir = dir.with_file_name(base);
            if let Ok(slide) = Self::get_slide_inner(&self.root_dir, &base_dir) {
                if slide.conf.version == version {
                    return Ok(slide);
                }
            }

            let archived_dir = base_dir.join(format!("v{version}"));
            if let Ok(slide) = Self::get_slide_inner(&self.root_dir, &archived_dir) {
                return Ok(slide);
            }
        }

        Self::get_slide_inner(&self.root_dir, dir)
    }

    /// get archived versions of a slide (src/<slide>/v*)
    pub fn get_archived_slides(&self, slide: &Slide) -> anyhow::Result<Vec<Slide>> {
        let archived = fs::read_dir(&slide.dir)?
            .filter_map(|e| e.ok())
            .filter(|entry| entry.path().is_dir())
            .filter(|entry| {
                entry.file_name().to_str().is_some_and(|name| {
                    name.starts_with('v') && name[1..].chars().all(|c| c.is_ascii_digit())
                })
            })
            .filter_map(|entry| Self::get_slide_inner(&self.root_dir, &entry.path()).ok())
            .sorted_by_key(|slide| slide.conf.version)
            .collect();

        Ok(archived)
    }
}

fn split_versioned_alias(path: &Path) -> Option<(String, u8)> {
    let file_name = path.file_name()?.to_str()?;
    let (base, version) = file_name.rsplit_once("_v")?;
    let version: u8 = version.parse().ok()?;
    if base.is_empty() {
        return None;
    }
    Some((base.to_string(), version))
}

#[cfg(test)]
mod tests {
    use crate::subcommand::{add::add, init::init, version::bump};

    use super::Project;

    #[test]
    fn test_get_slide_list_ignores_non_directory_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        std::fs::write(root.join("src/.DS_Store"), "").unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        assert_eq!(project.slides.len(), 0);
    }

    #[test]
    fn test_get_slide_accepts_versioned_alias() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false).unwrap();
        bump(
            &Project::get(root.to_path_buf()).unwrap(),
            "src/intro".into(),
        )
        .unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let latest = project
            .get_slide(std::path::Path::new("src/intro_v2"))
            .unwrap();
        assert_eq!(latest.conf.version, 2);

        let archived = project
            .get_slide(std::path::Path::new("src/intro_v1"))
            .unwrap();
        assert_eq!(archived.conf.version, 1);
        assert!(archived.dir.to_string_lossy().ends_with("src/intro/v1"));
    }
}
