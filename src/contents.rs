//! スライドの中身の操作

use anyhow::bail;
use regex::Regex;

/// contents of slide
#[derive(Debug)]
pub struct SlideContents {
    /// Frontmatter
    pub frontmatter: String,
    /// Pages of slide
    pub pages: Vec<SlidePage>,
}

impl TryFrom<&str> for SlideContents {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let separator = Regex::new(r"^-{3,}$").unwrap();

        let mut splitted = vec![];

        for line in value.lines() {
            if separator.is_match(line) {
                splitted.push(String::default());
            } else {
                // 末尾の文字列に追加
                if let Some(last) = splitted.last_mut() {
                    *last += line;
                    *last += "\n";
                } else {
                    bail!("Frontmatter is missing");
                }
            }
        }

        if splitted.len() < 2 {
            bail!("Frontmatter is missing");
        }

        Ok(Self {
            frontmatter: splitted.remove(0),
            pages: splitted
                .into_iter()
                .map(|s| SlidePage {
                    contents: s.trim().to_string(),
                })
                .collect(),
        })
    }
}

/// Pages of slide
#[derive(Debug)]
pub struct SlidePage {
    /// Contents of page
    contents: String,
}

#[cfg(test)]
mod test_contents {
    use super::*;

    #[test]
    fn test_slide_contents() {
        let s = r#"---
marp: true
title: Sample Slide
author: John Doe
---

# Slide 1
Some content here.

---

# Slide 2
More content here.

---
# Slide 3
Some content here.
"#;

        let slide_contents = SlideContents::try_from(s).unwrap();

        assert_eq!(
            &slide_contents.frontmatter,
            "marp: true\ntitle: Sample Slide\nauthor: John Doe\n"
        );

        assert_eq!(slide_contents.pages.len(), 3);
        assert_eq!(
            &slide_contents.pages[0].contents,
            &"# Slide 1\nSome content here."
        );
        assert_eq!(
            &slide_contents.pages[1].contents,
            &"# Slide 2\nMore content here."
        );
        assert_eq!(
            &slide_contents.pages[2].contents,
            &"# Slide 3\nSome content here."
        );
    }

    #[test]
    fn test_slide_contents_no_frontmatter() {
        let s = r#"# Slide 1
Some content here.
"#;

        let result = SlideContents::try_from(s);
        assert!(result.is_err());
    }
}
