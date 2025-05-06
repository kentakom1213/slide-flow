//! 新しくスライドを作成する

use std::fs;

use anyhow::bail;

use crate::{config::SlideConf, project::Project};

pub fn new(project: &Project, name: String, secret: bool, draft: bool) -> anyhow::Result<()> {
    // スライドのディレクトリ
    let slides_dir = project.root_dir.join("src").join(&name);

    if slides_dir.exists() {
        bail!("The slide already exists: {:?}", slides_dir);
    }

    // ディレクトリの作成
    fs::create_dir(&slides_dir)?;

    // 画像ディレクトリの作成
    let images_dir = slides_dir.join("images");
    fs::create_dir(&images_dir)?;
    fs::write(images_dir.join(".gitkeep"), "")?;

    // スライドの作成
    let slide_path = slides_dir.join("slide.md");
    fs::write(&slide_path, &project.conf.template.slide)?;

    // 設定ファイルの作成
    let conf = SlideConf {
        name,
        version: 1,
        secret: secret.then(|| uuid::Uuid::new_v4().to_string()),
        custom_path: Some(vec![]),
        draft: draft.then(|| true),
        description: Some(String::new()),
        title_prefix: None,
    };

    let conf_str = toml::to_string(&conf)?;
    let conf_path = slides_dir.join("slide.toml");

    fs::write(conf_path, conf_str)?;

    log::info!("Created a new slide: {:?}", slide_path);

    Ok(())
}
