use crate::{config::SlideType, project::Project};

pub fn list(project: &Project) -> anyhow::Result<()> {
    println!("{}", render(project));
    Ok(())
}

fn render(project: &Project) -> String {
    let rows = std::iter::once([
        "no".to_string(),
        "name".to_string(),
        "version".to_string(),
        "type".to_string(),
        "draft".to_string(),
    ])
    .chain(project.slides.iter().enumerate().map(|(idx, slide)| {
        [
            (idx + 1).to_string(),
            slide.conf.name.clone(),
            slide.conf.version.to_string(),
            slide_type_label(&slide.conf.type_).to_string(),
            slide.conf.draft.unwrap_or(false).to_string(),
        ]
    }))
    .collect::<Vec<_>>();

    let widths = (0..5)
        .map(|idx| {
            rows.iter()
                .map(|row| display_width(&row[idx]))
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    rows.iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .map(|(idx, value)| pad_cell(value, widths[idx]))
                .collect::<Vec<_>>()
                .join("  ")
                .trim_end()
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn slide_type_label(type_: &SlideType) -> &'static str {
    match type_ {
        SlideType::Marp => "marp",
        SlideType::Ipe => "ipe",
    }
}

fn display_width(value: &str) -> usize {
    value
        .chars()
        .map(|ch| if ch.is_ascii() { 1 } else { 2 })
        .sum()
}

fn pad_cell(value: &str, width: usize) -> String {
    let padding = width.saturating_sub(display_width(value));
    format!("{value}{}", " ".repeat(padding))
}

#[cfg(test)]
mod tests {
    use crate::{
        config::SlideType,
        project::Project,
        subcommand::{add::add, init::init},
    };

    use super::render;

    #[test]
    fn test_render_lists_slides_as_aligned_table() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        init(root).unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        add(&project, "alpha".to_string(), false, false, SlideType::Marp).unwrap();
        add(&project, "日本語".to_string(), false, true, SlideType::Ipe).unwrap();

        let project = Project::get(root.to_path_buf()).unwrap();
        let output = render(&project);

        assert_eq!(
            output,
            "no  name    version  type  draft\n1   alpha   1        marp  false\n2   日本語  1        ipe   true"
        );
    }
}
