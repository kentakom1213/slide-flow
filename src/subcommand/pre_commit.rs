//! project refresh and output cleanup helpers

use std::{collections::HashSet, fs, path::PathBuf};

use askama::Template;

use crate::{
    path::PublishPlan,
    project::Project,
    template::{IndexTemplate, PublishedSlide, ReadmeTemplate},
};

/// create list of slides
/// - index.html
/// - README.md
pub fn refresh_project_files(project: &Project) -> anyhow::Result<()> {
    // get slide configurations
    let slides = project
        .slides
        .iter()
        .map(|slide| PublishedSlide::from_slide(project, slide))
        .collect::<Vec<_>>();

    // generate README.md
    let readme_temp = ReadmeTemplate {
        project: &project.conf,
        slides: &slides,
    };

    // save README.md
    fs::write(project.root_dir.join("README.md"), readme_temp.render()?)?;

    log::info!("update: README.md");

    // generate index.html
    let index_temp = IndexTemplate { slides: &slides };
    let output_dir = project.root_dir.join(&project.conf.output_dir);
    fs::create_dir_all(&output_dir)?;

    // save index.html
    fs::write(output_dir.join("index.html"), index_temp.render()?)?;

    log::info!("update: {}/index.html", project.conf.output_dir);

    Ok(())
}

/// prune stale generated outputs
///
/// **input**
/// - `project`: project information
pub fn prune_stale_outputs(project: &Project, apply: bool) -> anyhow::Result<()> {
    let remove_files = stale_output_files(project)?;

    for file in remove_files {
        if apply {
            let remove_result = if file.is_dir() {
                fs::remove_dir_all(&file)
            } else {
                fs::remove_file(&file)
            };

            match remove_result {
                Ok(_) => println!("Removed: {}", file.to_string_lossy()),
                Err(e) => log::error!("failed to remove: {}, error: {}", file.to_string_lossy(), e),
            }
        } else {
            println!("Would remove: {}", file.to_string_lossy());
        }
    }

    Ok(())
}

pub fn stale_output_files(project: &Project) -> anyhow::Result<Vec<PathBuf>> {
    // files/directories not to be removed from output root
    let mut retained_files: HashSet<String> = HashSet::new();
    retained_files.insert("index.html".to_string());
    // `<output_dir>/images/optimized` is public output referenced by generated HTML,
    // not disposable internal cache. Keep the whole public images tree unless a
    // future pruner understands asset references precisely.
    retained_files.insert("images".to_string());

    for slide in &project.slides {
        if slide.conf.draft.unwrap_or(false) {
            continue;
        }

        let plan = PublishPlan::for_slide(project, slide);

        for stem in plan.html_stems {
            retained_files.insert(stem);
        }
        for file_name in plan.latest_pdf_aliases {
            retained_files.insert(file_name);
        }
        for alias in plan.alias_stems {
            retained_files.insert(alias);
        }
        for stem in plan.versioned_pdf_stems {
            retained_files.insert(stem + ".pdf");
        }

        for archived in project.get_archived_slides(slide)? {
            if archived.conf.draft.unwrap_or(false) {
                continue;
            }
            for stem in PublishPlan::for_slide(project, &archived).versioned_pdf_stems {
                retained_files.insert(stem + ".pdf");
            }
        }
    }

    // output directory
    let output_dir = project.root_dir.join(&project.conf.output_dir);
    if !output_dir.exists() {
        return Ok(vec![]);
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
        })
        .collect::<Vec<_>>();

    Ok(remove_files)
}

#[cfg(test)]
mod tests {
    use crate::{
        config::SlideType,
        project::Project,
        subcommand::{add::add, init::init},
    };

    use super::{prune_stale_outputs, stale_output_files};

    #[test]
    fn stale_output_files_retains_public_optimized_images() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false, SlideType::Marp).unwrap();

        let output_images = root.join("output").join("images").join("optimized");
        std::fs::create_dir_all(&output_images).unwrap();
        std::fs::write(output_images.join("example.png"), "fake").unwrap();
        std::fs::write(root.join("output").join("stale.html"), "").unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let stale = stale_output_files(&project).unwrap();

        assert!(stale.iter().any(|path| path.ends_with("stale.html")));
        assert!(!stale.iter().any(|path| path.ends_with("images")));
        assert!(!stale
            .iter()
            .any(|path| path.ends_with("images/optimized/example.png")));
    }

    #[test]
    fn prune_outputs_apply_retains_public_optimized_images() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();
        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "intro".to_string(), false, false, SlideType::Marp).unwrap();

        let optimized_image = root
            .join("output")
            .join("images")
            .join("optimized")
            .join("example.png");
        std::fs::create_dir_all(optimized_image.parent().unwrap()).unwrap();
        std::fs::write(&optimized_image, "fake").unwrap();
        let stale_file = root.join("output").join("stale.html");
        std::fs::write(&stale_file, "").unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        prune_stale_outputs(&project, true).unwrap();

        assert!(optimized_image.exists());
        assert!(!stale_file.exists());
    }
}
