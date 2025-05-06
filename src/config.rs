//! 設定ファイル

use serde::{Deserialize, Serialize};

/// プロジェクトの設定ファイル
#[derive(Debug, Deserialize)]
pub struct ProjectConf {
    /// プロジェクトの名前
    pub name: String,
    /// 著者名
    pub author: String,
    /// BASE_URL
    pub base_url: String,
    /// 出力ディレクトリ
    pub output_dir: String,
    /// テンプレートの設定
    pub template: TemplateConf,
    /// ビルドの設定
    pub build: BuildConf,
}

/// テンプレートの設定
#[derive(Debug, Deserialize)]
pub struct TemplateConf {
    /// スライドのテンプレート
    pub slide: String,
    /// インデックスページのテンプレート
    pub index: String,
    /// スライドの末尾につける文字列
    pub suffix: String,
}

/// ビルド用の設定
#[derive(Debug, Deserialize)]
pub struct BuildConf {
    /// テーマのディレクトリ
    pub theme_dir: String,
    /// `marp`の実行ファイル
    pub marp_binary: String,
}

/// スライドの設定ファイル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideConf {
    /// スライドの名前
    pub name: String,
    /// スライドのバージョン
    pub version: u8,
    /// 限定公開の場合のUUID
    pub secret: Option<String>,
    /// スライドのパス
    pub custom_path: Option<Vec<String>>,
    /// 公開するか
    pub draft: Option<bool>,
    /// 解説
    pub description: Option<String>,
    /// スライドの見出しの区切り文字
    pub title_prefix: Option<String>,
}

#[cfg(test)]
mod test_config {
    use super::*;

    #[test]
    fn test_parse_project_config() {
        let config_example = r##"
            name = "slide-flow"
            author = "powell"
            base_url = "https://test.dev/"
            output_dir = "output"
            
            [template]
            slide = "<!-- slide -->"
            index = "<!-- index -->"
            suffix = "<!-- slide-end -->"

            [build]
            theme_dir = ".marp/themes"
            marp_binary = "marp"
        "##;

        let config: ProjectConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);
    }

    #[test]
    fn test_parse_slide_config() {
        let config_example = r###"
            version = 1
            name = "slide1"
            path = "slide1"
            draft = true
            description = "This is slide1"
            title_prefix = "##"
        "###;

        let config: SlideConf = toml::from_str(&config_example).unwrap();

        println!("{:#?}", config);
    }
}
