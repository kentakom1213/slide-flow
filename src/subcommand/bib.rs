use crate::slide::Slide;

/// modify bibliography
pub fn update_bibliography(target_slide: Slide) -> anyhow::Result<()> {
    let mut contents = target_slide.get_contents()?;

    log::info!(
        "Updating bibliography in slide: {}",
        target_slide.slide_path().to_string_lossy()
    );

    let bib_entries = target_slide.conf.bibliography.as_deref().unwrap_or(&[]);

    // modify bibliography
    contents.modify_bibliography(bib_entries);

    log::info!(
        "Modified bibliography in slide: {}",
        target_slide.slide_path().to_string_lossy()
    );

    // new slide string
    let new_contents = contents.to_marp();

    // save to slide file
    std::fs::write(target_slide.slide_path(), new_contents)?;

    log::info!(
        "Saved slide file: {}",
        target_slide.slide_path().to_string_lossy()
    );

    anyhow::Ok(())
}
