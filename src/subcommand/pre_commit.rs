//! pre-commit操作を行うサブコマンド

use std::{collections::HashSet, fs};

use askama::Template;

use crate::{
    project::Project,
    template::{IndexTemplate, ReadmeTemplate},
};

/// ファイルを作成する
/// - index.html
/// - README.md
pub fn create_files(project: &Project) -> anyhow::Result<()> {
    // スライドの設定一覧を取得
    let slide_configs = project.get_slide_conf_list();

    // README.mdのレンダリング
    let readme_temp = ReadmeTemplate {
        project: &project.conf,
        slides: &slide_configs,
    };

    // 保存
    fs::write(project.root_dir.join("README.md"), readme_temp.render()?)?;

    log::info!("update: README.md");

    // index.htmlのレンダリング
    let index_temp = IndexTemplate {
        slides: &slide_configs,
    };

    // 保存
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

/// 過去のビルド情報を削除
///
/// **引数**
/// - `project`: プロジェクト情報
pub fn remove_cache(project: &Project) -> anyhow::Result<()> {
    // 残すファイル名
    let retained_files: HashSet<_> = project
        .get_slide_conf_list()
        .iter()
        // ビルド対象のスライドは除外
        .filter(|conf| !conf.draft.unwrap_or(false))
        .flat_map(|conf| {
            [conf.secret.to_owned().unwrap_or(conf.name.to_owned())]
                .into_iter()
                // custom_pathが指定されている場合，そちらも反映
                .chain(
                    conf.custom_path
                        .as_ref()
                        .into_iter()
                        .flat_map(|s| s.to_owned()),
                )
        })
        .collect();

    // 出力ディレクトリ以下のファイルを削除
    let output_dir = project.root_dir.join(&project.conf.output_dir);

    // 削除対象のディレクトリを取得
    let remove_files = fs::read_dir(output_dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| {
            let stem = path.file_stem()?.to_str()?;
            // 含まれないファイルを削除対象とする
            (!retained_files.contains(stem)).then_some(path)
        });

    // 削除対象のファイルを削除
    for file in remove_files {
        // 削除
        let remove_result = if file.is_dir() {
            fs::remove_dir_all(&file)
        } else {
            fs::remove_file(&file)
        };

        match remove_result {
            Ok(_) => log::info!("remove: {:?}", file),
            Err(e) => log::error!("failed to remove: {:?}, error: {}", file, e),
        }
    }

    Ok(())
}
