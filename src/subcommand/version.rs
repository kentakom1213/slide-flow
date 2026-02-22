use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};

use crate::{project::Project, slide::SlideType};

pub fn bump(project: &Project, dir: PathBuf) -> anyhow::Result<()> {
    if is_version_dir(&dir) {
        bail!("Please specify a slide root directory, not a version directory: {dir:?}");
    }

    let slide = project.get_slide(&dir)?;
    let current_version = slide.conf.version;
    let archive_dir = slide.dir.join(format!("v{current_version}"));

    if archive_dir.exists() {
        bail!(
            "Archive directory already exists: {}",
            archive_dir.to_string_lossy()
        );
    }

    fs::create_dir(&archive_dir)?;

    let slide_file_name = match slide.type_ {
        SlideType::Marp => "slide.md",
        SlideType::Ipe => "slide.ipe",
    };

    let slide_file = slide.dir.join(slide_file_name);
    let conf_file = slide.dir.join("slide.toml");
    let images_dir = slide.dir.join("images");

    copy_required_file(&slide_file, &archive_dir.join(slide_file_name))?;
    copy_required_file(&conf_file, &archive_dir.join("slide.toml"))?;

    if images_dir.exists() {
        copy_dir_all(&images_dir, &archive_dir.join("images"))?;
    }

    fs::remove_file(&slide_file)?;
    fs::remove_file(&conf_file)?;
    if images_dir.exists() {
        fs::remove_dir_all(&images_dir)?;
    }

    fs::create_dir_all(&images_dir)?;
    fs::write(images_dir.join(".gitkeep"), "")?;

    match slide.type_ {
        SlideType::Marp => fs::write(slide.dir.join("slide.md"), &project.conf.template.slide)?,
        SlideType::Ipe => fs::write(slide.dir.join("slide.ipe"), "")?,
    }

    let mut new_conf = slide.conf.clone();
    new_conf.version = current_version
        .checked_add(1)
        .context("version overflow while bumping")?;
    let conf_str = toml::to_string(&new_conf)?;
    fs::write(slide.dir.join("slide.toml"), conf_str)?;

    log::info!("archived: {}", archive_dir.to_string_lossy());
    log::info!("bumped: {} -> {}", current_version, new_conf.version);

    Ok(())
}

fn copy_required_file(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        bail!("Required file does not exist: {}", src.to_string_lossy());
    }

    fs::copy(src, dst)?;

    if !dst.exists() {
        bail!("Failed to verify copied file: {}", dst.to_string_lossy());
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.exists() {
        return Ok(());
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn is_version_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    name.starts_with('v') && name[1..].chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{
        project::Project,
        subcommand::{add::add, init::init},
    };

    use super::bump;

    #[test]
    fn test_bump_archives_and_increments_version() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false).unwrap();
        std::fs::write(root.join("src/intro/slide.md"), "# before bump").unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        bump(&project, PathBuf::from("src/intro")).unwrap();

        let archived_md = root.join("src/intro/v1/slide.md");
        assert!(archived_md.exists());
        assert_eq!(
            std::fs::read_to_string(archived_md).unwrap(),
            "# before bump"
        );
        assert!(root.join("src/intro/v1/slide.toml").exists());
        assert!(root.join("src/intro/v1/images/.gitkeep").exists());

        let conf: crate::config::SlideConf =
            toml::from_str(&std::fs::read_to_string(root.join("src/intro/slide.toml")).unwrap())
                .unwrap();
        assert_eq!(conf.version, 2);
        assert!(root.join("src/intro/images/.gitkeep").exists());
    }

    #[test]
    fn test_bump_fails_if_archive_already_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false).unwrap();
        std::fs::create_dir_all(root.join("src/intro/v1")).unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let res = bump(&project, PathBuf::from("src/intro"));
        assert!(res.is_err());
    }
}
