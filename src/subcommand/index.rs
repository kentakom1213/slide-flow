//! スライドのインデックスを作成する

use std::fs;

use anyhow::bail;
use regex::Regex;

use crate::slide::Slide;

/// スライドにインデックスを付ける
pub fn put_index(slide: &Slide) -> anyhow::Result<String> {
    // Markdownファイルのパス
    let slide_path = slide.slide_path();

    // スライドを読み込み，行ごとに分割
    let Ok(mut lines) =
        fs::read_to_string(&slide_path).map(|s| s.lines().map(String::from).collect::<Vec<_>>())
    else {
        bail!("The slide file does not exist: {:?}", slide_path);
    };

    // 見出しの区切り文字
    let title_prefix: &str = slide.conf.title_prefix.as_deref().unwrap_or("# ");

    // 番号を表す正規表現
    let slide_number = Regex::new(r"\(\d+/\d+\)$").unwrap();

    // レベル1の見出しを取得
    let mut toc = String::new();

    let mut titles = lines
        .iter()
        .enumerate()
        .filter(|&(_, line)| line.starts_with(title_prefix))
        .map(|(i, line)| {
            let title = slide_number.replace(&line, "").trim().to_string();
            (i, title)
        })
        // ランレングス圧縮
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
        // インデックスとタイトルのペアにする
        .flat_map(|(idxs, title)| {
            // 目次に追加
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

    // 書き換え
    lines.iter_mut().enumerate().for_each(|(i, line)| {
        if titles.peek().is_some_and(|(l, _)| *l == i) {
            let (_, title) = titles.next().unwrap();
            *line = title;
        }
    });

    lines.push(String::new());

    // 書き込み
    let Ok(_) = fs::write(&slide_path, lines.join("\n")) else {
        bail!("Failed to write the slide file: {:?}", slide_path);
    };

    // 目次を返す
    Ok(toc)
}
