use std::{fs, path::PathBuf};

use anyhow::bail;
use itertools::Itertools;

use crate::{
    config::{ProjectConf, SlideConf},
    slide::Slide,
};

/// プロジェクトの情報
#[derive(Debug)]
pub struct Project {
    /// `config.toml`が配置されているディレクトリ
    pub root_dir: PathBuf,
    /// 設定
    pub conf: ProjectConf,
    /// スライド一覧
    pub slides: Vec<Slide>,
}

impl Project {
    /// プロジェクトの情報を取得する
    pub fn get(root_dir: PathBuf) -> anyhow::Result<Self> {
        // プロジェクトの設定ファイルを読み込む
        let conf_path = root_dir.join("config.toml");
        let Ok(conf_str) = std::fs::read_to_string(&conf_path) else {
            bail!("The project config file does not exist: {:?}", conf_path);
        };
        let conf: ProjectConf = toml::from_str(&conf_str)?;
        // スライド一覧
        let slides = Self::get_slide_list(&root_dir)?;

        Ok(Self {
            root_dir,
            conf,
            slides,
        })
    }

    /// スライド一覧を取得する
    fn get_slide_list(root_dir: &PathBuf) -> anyhow::Result<Vec<Slide>> {
        // 各ファイルに対して操作を行う
        let slides = fs::read_dir(root_dir.join("src"))?
            .filter_map(|e| e.ok())
            .filter_map(|dir| Self::get_slide_inner(&root_dir, &dir.path()).ok())
            // 名前順にソート
            .sorted_by(|a, b| a.dir.cmp(&b.dir))
            .collect::<Vec<_>>();

        Ok(slides)
    }

    /// スライドのディレクトリを取得する
    fn get_slide_inner(root_dir: &PathBuf, dir: &PathBuf) -> anyhow::Result<Slide> {
        // ディレクトリ
        let dir = root_dir.join(&dir);
        // プロジェクトの設定ファイルを読み込む
        let conf_path = dir.join("slide.toml");
        // スライドの設定ファイルを読み込む
        let Ok(conf_str) = std::fs::read_to_string(&conf_path) else {
            bail!("The project config file does not exist: {:?}", conf_path);
        };
        let conf: SlideConf = toml::from_str(&conf_str)?;

        Ok(Slide { dir, conf })
    }

    /// スライドの設定ファイル一覧を取得する
    pub fn get_slide_conf_list(&self) -> Vec<SlideConf> {
        self.slides.iter().map(|slide| slide.conf.clone()).collect()
    }

    /// 特定のスライドを取得する
    pub fn get_slide(&self, dir: &PathBuf) -> anyhow::Result<Slide> {
        Self::get_slide_inner(&self.root_dir, dir)
    }
}
