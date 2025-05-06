//! ローカルでビルドを行う

use std::{path::PathBuf, sync::Arc};

use colored::Colorize;
use tokio::{process::Command, runtime::Runtime, sync::Semaphore};

use crate::{config::SlideConf, project::Project, slide::Slide};

use super::util::copy_images;

/// ビルドコマンドのとその情報を保持する構造体
pub enum BuildCommand {
    /// PDFのビルドコマンド
    PDF {
        /// ビルド対象のディレクトリ
        dir: PathBuf,
        /// ビルドコマンド
        command: Command,
        /// 設定
        conf: SlideConf,
    },
    /// HTMLのビルドコマンド
    HTML {
        /// ビルド対象のディレクトリ
        dir: PathBuf,
        /// ビルドコマンド
        command: Command,
        /// 設定
        conf: SlideConf,
    },
}

/// ビルドコマンドを実行する
pub fn build<'a>(commands: impl Iterator<Item = BuildCommand>, max_concurrent: usize) {
    // Tokio ランタイムを手動で作成
    let runtime = Runtime::new().unwrap();

    // ビルドコマンドを実行
    runtime.block_on(async {
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        let handles: Vec<_> = commands
            .into_iter()
            .filter(|cmd| match cmd {
                BuildCommand::HTML { conf, .. } => !conf.draft.unwrap_or(false),
                BuildCommand::PDF { conf, .. } => !conf.draft.unwrap_or(false),
            })
            .map(|cmd| {
                let (dir, build_type, mut command) = match cmd {
                    BuildCommand::PDF { dir, command, .. } => (dir, "PDF", command),
                    BuildCommand::HTML { dir, command, .. } => (dir, "HTML", command),
                };

                let semaphore = semaphore.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire_owned().await.unwrap();

                    match command.output().await {
                        Ok(_) => {
                            log::info!("build {}: {:?} ... {}", build_type, dir, "done".green());
                        }
                        Err(e) => {
                            log::error!("build {}: {:?} ... {}", build_type, dir, "failed".red());
                            log::error!("error: {:?}", e);
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.await.unwrap();
        }
    });
}

/// 出力ファイル名のstemを生成
fn make_file_stems(slide: &Slide) -> Vec<String> {
    let mut res = slide.conf.custom_path.clone().unwrap_or_default();

    if let Some(prefix) = &slide.conf.secret {
        res.push(prefix.clone());
    } else {
        res.push(slide.conf.name.clone());
    }

    res
}

/// PDFファイルのビルドを行うコマンドを生成する
pub fn build_pdf_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    // コマンドを生成するクロージャ
    let make_commmand = move |output_stem: String| {
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd
            // テーマの指定
            .arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            // htmlを有効化
            .arg("--html")
            .arg("true")
            // 出力先の指定
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(output_stem)
                    .with_extension("pdf"),
            )
            // PDFの出力
            .arg("--pdf")
            // ディレクトリの指定
            .arg("--input-dif")
            .arg(&slide.dir)
            .arg("--allow-local-files")
            // タイトル
            .arg("--title")
            .arg(&slide.conf.name)
            // 著者
            .arg("--author")
            .arg(&project.conf.author)
            // 説明
            .arg("--description")
            .arg(&slide.conf.description.clone().unwrap_or_else(String::new))
            // 入力となるマークダウンファイル
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    // 出力ファイル名を生成
    let output_files = make_file_stems(&slide);

    output_files.into_iter().map(move |stem| BuildCommand::PDF {
        dir: slide.dir.clone(),
        command: make_commmand(stem),
        conf: slide.conf.clone(),
    })
}

/// HTMLファイルのビルドを行うコマンドを生成する
pub fn build_html_commands<'a>(
    project: &'a Project,
    slide: &'a Slide,
) -> impl Iterator<Item = BuildCommand> + 'a {
    // コマンドを生成するクロージャ
    let make_commmand = move |output_stem: String| {
        // ビルドを行うコマンド
        let mut cmd = Command::new(&project.conf.build.marp_binary);

        cmd
            // テーマの指定
            .arg("--theme-set")
            .arg(&project.conf.build.theme_dir)
            // htmlを有効化
            .arg("--html")
            .arg("true")
            // 出力先の指定
            .arg("-o")
            .arg(
                project
                    .root_dir
                    .join(&project.conf.output_dir)
                    .join(&output_stem)
                    .join("index.html"),
            )
            // タイトル
            .arg("--title")
            .arg(&slide.conf.name)
            // 著者
            .arg("--author")
            .arg(&project.conf.author)
            // 説明
            .arg("--description")
            .arg(&slide.conf.description.clone().unwrap_or_else(String::new))
            // 入力となるマークダウンファイル
            .arg(slide.dir.join("slide.md"));

        cmd
    };

    // 出力ファイル名を生成
    let output_files = make_file_stems(&slide);

    output_files
        .into_iter()
        .map(move |stem| BuildCommand::HTML {
            dir: slide.dir.clone(),
            command: make_commmand(stem),
            conf: slide.conf.clone(),
        })
}

/// HTMLのために画像をコピー
pub fn copy_images_html<'a>(project: &'a Project, slide: &'a Slide) -> anyhow::Result<()> {
    let output_files = make_file_stems(slide);

    for stem in output_files {
        let target_images_dir = project
            .root_dir
            .join(&project.conf.output_dir)
            .join(&stem)
            .join("images");

        // ディレクトリの作成（存在しない場合，クリア）
        if target_images_dir.exists() {
            std::fs::remove_dir_all(&target_images_dir)?;
        }
        std::fs::create_dir_all(&target_images_dir)?;

        // 画像のコピー
        copy_images(&slide, &target_images_dir)?;
    }

    Ok(())
}
