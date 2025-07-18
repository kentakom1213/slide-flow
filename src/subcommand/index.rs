//! put index to slide

use std::fs;

use anyhow::bail;
use regex::Regex;

use crate::slide::Slide;

/// put index to slide and return the table of contents
pub fn put_index(slide: &Slide) -> anyhow::Result<String> {
    // path to slide
    let slide_path = slide.slide_path();

    // read lines of slide file
    let Ok(mut lines) =
        fs::read_to_string(&slide_path).map(|s| s.lines().map(String::from).collect::<Vec<_>>())
    else {
        bail!(
            "The slide file does not exist: {}",
            slide_path.to_string_lossy()
        );
    };

    // prefix of slide title
    let title_prefix: &str = slide.conf.title_prefix.as_deref().unwrap_or("# ");

    // regex for slide number
    let slide_number = Regex::new(r"\(\d+/\d+\)$").unwrap();

    // get slide titles
    let mut toc = String::new();

    let mut titles = lines
        .iter()
        .enumerate()
        .filter(|&(_, line)| line.starts_with(title_prefix))
        .map(|(i, line)| {
            let title = slide_number.replace(line, "").trim().to_string();
            (i, title)
        })
        // run length encoding
        .fold(vec![], |mut acc: Vec<(Vec<usize>, String)>, (i, title)| {
            if let Some((idxs, last_title)) = acc.last_mut() {
                if title == *last_title {
                    idxs.push(i);
                } else {
                    acc.push((vec![i], title));
                }
            } else {
                acc.push((vec![i], title));
            }
            acc
        })
        .into_iter()
        // make pair of slide number and title
        .flat_map(|(idxs, title)| {
            // add to table of contents
            toc.push_str(&format!(
                "1. {}\n",
                title.trim_start_matches(title_prefix).trim()
            ));

            let n = idxs.len();
            if n > 1 {
                let itr = idxs
                    .into_iter()
                    .enumerate()
                    .map(move |(i, l)| (l, format!("{} ({}/{})", title, i + 1, n)));

                either::Left(itr)
            } else {
                let itr = idxs.into_iter().map(move |l| (l, title.clone()));

                either::Right(itr)
            }
        })
        .peekable();

    // put slide number
    lines.iter_mut().enumerate().for_each(|(i, line)| {
        if titles.peek().is_some_and(|(l, _)| *l == i) {
            let (_, title) = titles.next().unwrap();
            *line = title;
        }
    });

    lines.push(String::new());

    // write for file
    let Ok(_) = fs::write(&slide_path, lines.join("\n")) else {
        bail!(
            "Failed to write the slide file: {}",
            slide_path.to_string_lossy()
        );
    };

    // output toc
    Ok(toc)
}
