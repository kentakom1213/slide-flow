use clap::Parser;
use slide_flow::{
    parser::{
        Cmd,
        SubCommands::{Build, Index, New, PreCommit},
    },
    project::Project,
    subcommand::{
        build::{build, build_html_commands, build_pdf_commands, copy_images_html},
        index::put_index,
        new::new,
        pre_commit::{create_files, remove_cache},
    },
};
use std::io::Write;

fn init_logger() {
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(buf, "[{}] {}", record.level(), record.args()) // ログレベルとメッセージのみ表示
        })
        .filter(None, log::LevelFilter::Trace)
        .init();
}

fn runner() -> anyhow::Result<()> {
    // ロガーの初期化
    init_logger();

    // カレントディレクトリの取得
    let root_dir = std::env::current_dir()?;

    // コマンドのパース
    let parser = Cmd::parse();

    // プロジェクトの情報を取得
    let project = Project::get(root_dir)?;

    // コマンドの実行
    match parser.subcommand {
        New {
            name,
            secret,
            draft,
        } => new(&project, name, secret, draft),
        PreCommit => {
            // キャッシュを削除
            remove_cache(&project)?;
            // スライドの一覧を生成
            create_files(&project)
        }
        Index { dir, quiet } => {
            if let Some(dir) = dir {
                let target_slide = project.get_slide(&dir)?;

                let toc = put_index(&target_slide)?;

                if !quiet {
                    println!("{toc}");
                }

                Ok(())
            } else {
                project
                    .slides
                    .iter()
                    .inspect(|slide| log::info!("Put index to slide: {:?}", slide.dir))
                    .map(|slide| put_index(slide).map(|_| ()))
                    .collect::<anyhow::Result<()>>()
            }
        }
        Build {
            directories,
            concurrent,
        } => {
            // コマンド群の生成
            let mut cmds = vec![];

            for dir in directories {
                let Ok(target_slide) = project.get_slide(&dir) else {
                    log::error!("The slide does not exist: {:?}", &dir);
                    continue;
                };

                let build_html_cmd = build_html_commands(&project, &target_slide);
                let build_pdf_cmd = build_pdf_commands(&project, &target_slide);

                // 画像ファイルのコピー
                if let Err(e) = copy_images_html(&project, &target_slide) {
                    log::error!("Failed to copy images: {:?}", e);
                    continue;
                }

                cmds.extend(build_html_cmd);
                cmds.extend(build_pdf_cmd);
            }

            build(cmds.into_iter(), concurrent);

            Ok(())
        }
    }
}

fn main() {
    // メイン処理
    let res = runner();

    // エラー処理
    if let Err(e) = res {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
