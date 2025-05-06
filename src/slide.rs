use std::path::PathBuf;

use crate::config::SlideConf;

/// プロジェクトの情報
#[derive(Debug, Clone)]
pub struct Slide {
    /// `slide.toml`が配置されているディレクトリ
    pub dir: PathBuf,
    /// 設定
    pub conf: SlideConf,
}

impl Slide {
    /// スライドのパスを取得する
    pub fn slide_path(&self) -> PathBuf {
        self.dir.join("slide.md")
    }

    /// 画像のディレクトリを取得する
    pub fn image_dir(&self) -> PathBuf {
        self.dir.join("images")
    }
}
