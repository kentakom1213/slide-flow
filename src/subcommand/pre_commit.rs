//! subcommand for pre-commit

use std::{collections::HashSet, fs};

use askama::Template;

use crate::{
    project::Project,
    subcommand::build::{make_file_stems, make_versioned_stems},
    template::{IndexTemplate, ReadmeTemplate},
};

/// create list of slides
/// - index.html
/// - README.md
pub fn create_files(project: &Project) -> anyhow::Result<()> {
    // get slide configurations
    let slide_configs = project.get_slide_conf_list();

    // generate README.md
    let readme_temp = ReadmeTemplate {
        project: &project.conf,
        slides: &slide_configs,
    };

    // save README.md
    fs::write(project.root_dir.join("README.md"), readme_temp.render()?)?;

    log::info!("update: README.md");

    // generate index.html
    let index_temp = IndexTemplate {
        slides: &slide_configs,
    };

    // save index.html
    fs::write(
        project
            .root_dir
            .join(&project.conf.output_dir)
            .join("index.html"),
        index_temp.render()?,
    )?;

    log::info!("update: {}/index.html", project.conf.output_dir);

    Ok(())
}

/// remove cache files
///
/// **input**
/// - `project`: project information
pub fn remove_cache(project: &Project) -> anyhow::Result<()> {
    // files/directories not to be removed from output root
    let mut retained_files: HashSet<String> = HashSet::new();
    retained_files.insert("index.html".to_string());

    for slide in &project.slides {
        if slide.conf.draft.unwrap_or(false) {
            continue;
        }

        for stem in make_versioned_stems(slide) {
            retained_files.insert(stem.clone());
            retained_files.insert(stem + ".pdf");
        }
        for stem in make_file_stems(slide) {
            retained_files.insert(stem + ".pdf");
        }

        for archived in project.get_archived_slides(slide)? {
            if archived.conf.draft.unwrap_or(false) {
                continue;
            }
            for stem in make_versioned_stems(&archived) {
                retained_files.insert(stem + ".pdf");
            }
        }
    }

    // output directory
    let output_dir = project.root_dir.join(&project.conf.output_dir);
    if !output_dir.exists() {
        return Ok(());
    }

    // get all files for removal
    let remove_files = fs::read_dir(output_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
                return false;
            };
            !retained_files.contains(name)
        });

    // remove files
    for file in remove_files {
        // remove
        let remove_result = if file.is_dir() {
            fs::remove_dir_all(&file)
        } else {
            fs::remove_file(&file)
        };

        match remove_result {
            Ok(_) => log::info!("remove: {}", file.to_string_lossy()),
            Err(e) => log::error!("failed to remove: {}, error: {}", file.to_string_lossy(), e),
        }
    }

    Ok(())
}
