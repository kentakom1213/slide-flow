//! subcommand for pre-commit

use std::{collections::HashSet, fs};

use askama::Template;

use crate::{
    project::Project,
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
    // directories not to be removed
    let retained_files: HashSet<_> = project
        .get_slide_conf_list()
        .iter()
        // retain only the files that are not draft
        .filter(|conf| !conf.draft.unwrap_or(false))
        .flat_map(|conf| {
            [conf.secret.to_owned().unwrap_or(conf.name.to_owned())]
                .into_iter()
                // If there is a custom path, it is retained as well.
                .chain(
                    conf.custom_path
                        .as_ref()
                        .into_iter()
                        .flat_map(|s| s.to_owned()),
                )
        })
        .collect();

    // output directory
    let output_dir = project.root_dir.join(&project.conf.output_dir);

    // get all files for removal
    let remove_files = fs::read_dir(output_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| {
            let stem = path.file_stem()?.to_str()?;
            // if the file is not in the retained files, it should be removed
            (!retained_files.contains(stem)).then_some(path)
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
