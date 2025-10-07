//! スライドの中身の操作

use std::collections::HashMap;

use anyhow::bail;
use itertools::Itertools;
use regex::Regex;

use crate::config::BibEntry;

/// contents of slide
#[derive(Debug)]
pub struct SlideContents {
    /// Frontmatter
    pub frontmatter: String,
    /// Pages of slide
    pub pages: Vec<SlidePage>,
}

impl SlideContents {
    /// update references in all pages
    pub fn modify_bibliography(&mut self, bib_entries: &[BibEntry]) {
        let entries_each_page = self.enumerate_bib_entries(bib_entries);
        let bib_index = self.generate_bib_index(bib_entries);

        for ((page, references), page_id) in self.pages.iter_mut().zip(entries_each_page).zip(1..) {
            // update references
            page.update_references(page_id, &bib_index);

            // update bibliography
            page.update_bib_footnote(page_id, &references, &bib_index)
        }
    }

    /// generate inverted index of bibliography entries
    pub fn generate_bib_index<'a>(
        &self,
        bib_entries: &'a [BibEntry],
    ) -> HashMap<&'a BibEntry, usize> {
        let mut bib_index = HashMap::new();

        for page in &self.pages {
            let refs = page.enumerate_references(bib_entries);
            for entry in refs {
                let cnt = bib_index.len();
                bib_index.entry(entry).or_insert(cnt + 1);
            }
        }

        bib_index
    }

    /// get bib entries each page
    pub fn enumerate_bib_entries<'a>(&self, bib_entries: &'a [BibEntry]) -> Vec<Vec<&'a BibEntry>> {
        self.pages
            .iter()
            .map(|page| page.enumerate_references(bib_entries))
            .collect()
    }

    /// encode to marp
    pub fn to_marp(&self) -> String {
        let mut s = String::new();
        s += "---\n";
        s += &self.frontmatter;
        s += "---\n";

        s += &self
            .pages
            .iter()
            .map(|p| p.contents.trim())
            .join("\n\n---\n\n");

        s
    }
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

impl SlidePage {
    /// enumerate references in the page
    pub fn enumerate_references<'a>(&self, bib_entries: &'a [BibEntry]) -> Vec<&'a BibEntry> {
        let re = Regex::new(r"\[.*?\]\(#(.*?)(|:\d+)\)").unwrap();

        // collect keys from the page
        let keys: Vec<&str> = re
            .captures_iter(&self.contents)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
            .collect();

        // find corresponding BibEntry for each key
        keys.iter()
            .filter_map(|&key| bib_entries.iter().find(|entry| entry.tag == key))
            .unique()
            .collect()
    }

    /// update reference item
    pub fn update_references(&mut self, page_id: usize, bib_index: &HashMap<&BibEntry, usize>) {
        let re = Regex::new(r"\[.*?\]\(#(.*?)(|:\d+)\)").unwrap();

        let new_contents = re
            .replace_all(&self.contents, |caps: &regex::Captures| {
                let key = &caps[1];
                if let Some(entry) = bib_index.keys().find(|e| e.tag == key) {
                    if let Some(&idx) = bib_index.get(entry) {
                        format!("[{idx}](#{key}:{page_id})")
                    } else {
                        caps[0].to_string()
                    }
                } else {
                    caps[0].to_string()
                }
            })
            .to_string();

        self.contents = new_contents;
    }

    /// update reference of footnote
    pub fn update_bib_footnote(
        &mut self,
        page_id: usize,
        references: &[&BibEntry],
        bib_index: &HashMap<&BibEntry, usize>,
    ) {
        if references.is_empty() {
            return;
        }

        let new_bibliography = Self::generate_new_bibliography(page_id, references, bib_index);

        let re = Regex::new(r#"(?s)<div class="footnote">(.*?)</div>"#).unwrap();

        if let Some((fstart, fend)) = re
            .captures(&self.contents)
            .and_then(|cap| cap.get(1))
            .map(|m| (m.start(), m.end()))
        {
            let old_footnote = &self.contents[fstart..fend];

            // clear references
            let re = Regex::new(r#"(?m)<span id=".*">.*</span>.*"#).unwrap();
            let mut new_footnote = re.replace_all(old_footnote, "").to_string();

            new_footnote += &new_bibliography;

            // replace old footnote
            self.contents.replace_range(fstart..fend, &new_footnote);
        } else {
            // append new footnote

            self.contents += &format!(
                r#"

<div class="footnote">

{new_bibliography}

</div>
"#
            );
        }
    }

    fn generate_new_bibliography(
        page_id: usize,
        references: &[&BibEntry],
        bib_index: &HashMap<&BibEntry, usize>,
    ) -> String {
        references
            .iter()
            .map(|e| (bib_index[e], e))
            .sorted_by_key(|(k, _)| *k)
            .map(|(k, e)| {
                format!(
                    r#"<span id="{}:{}">[{}]</span> {}"#,
                    e.tag,
                    page_id,
                    k,
                    e.format()
                )
            })
            .join("\n")
            + "\n\n"
    }
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

    #[test]
    fn test_enumerate_references() {
        let bib = vec![
            BibEntry {
                tag: "ref1".to_string(),
                authors: Some("Author A".to_string()),
                title: "Title A".to_string(),
                year: 2020,
                venue: Some("Venue A".to_string()),
                url: Some("https://doi.org/xxxx".to_string()),
            },
            BibEntry {
                tag: "ref2".to_string(),
                authors: Some("Author B".to_string()),
                title: "Title B".to_string(),
                year: 2021,
                venue: Some("Venue B".to_string()),
                url: Some("https://doi.org/yyyy".to_string()),
            },
        ];

        let s = r#"---
marp: true
title: Sample Slide
author: John Doe
---
# Slide 1
Some content here with a reference [see this](#ref2) and another [example](#ref1).
"#;

        let slide_contents = SlideContents::try_from(s).unwrap();
        let page = &slide_contents.pages[0];

        let refs = page.enumerate_references(&bib);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].tag, "ref2");
        assert_eq!(refs[1].tag, "ref1");
    }
}
