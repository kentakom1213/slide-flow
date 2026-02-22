# 設計

## 全体構成

`slide-flow` は次の層で構成されています。

- CLI 層: `src/parser.rs`（コマンド/引数定義）
- 実行分岐層: `src/main.rs`（サブコマンドごとの処理分岐）
- ドメイン層: `src/project.rs`, `src/slide.rs`, `src/config.rs`, `src/contents.rs`
- ユースケース層: `src/subcommand/*.rs`
- テンプレート層: `src/template.rs`, `templates/*`

## ディレクトリと責務

- `src/main.rs`
- CLI エントリポイント。`Project` をロードし、各サブコマンドを実行。

- `src/parser.rs`
- `clap` でコマンド定義。
- コマンドは `Init`, `Add`, `PreCommit`, `Index`, `Bib`, `Build`。

- `src/config.rs`
- `config.toml` / `slide.toml` のデータ構造定義。
- `ProjectConf`, `SlideConf`, `BibEntry` を保持。

- `src/project.rs`
- プロジェクト全体の読み込み。
- ルート `config.toml` を読み、`src/*` 配下からスライド一覧を構築。

- `src/slide.rs`
- 1スライドを表すモデル。
- 種別（Marp/Ipe）判定、スライド本体と画像ディレクトリへのパス提供。

- `src/contents.rs`
- Marp テキストをページ単位で扱うロジック。
- 参考文献リンク（`[n](#tag)`）の正規化、脚注ブロック更新を実装。

- `src/subcommand/*.rs`
- `init`: 初期ディレクトリと既定 `config.toml` 作成
- `add`: スライドディレクトリ/`slide.md`/`slide.toml` 作成
- `index`: タイトル行へ連番を振り、目次テキストを返す
- `bib`: 文献参照と脚注を更新して `slide.md` へ保存
- `pre_commit`: `README.md` と `output/index.html` を再生成し不要成果物を削除
- `build`: Marp コマンド生成・並列実行、画像コピー、Ipe PDF コピー

- `templates/readme.md`, `templates/index.html`
- Askama テンプレート。公開スライド一覧をレンダリング。

## 主要データモデル

- `ProjectConf`
- プロジェクト名、著者、`base_url`、`output_dir`、テンプレート設定、ビルド設定を管理。

- `SlideConf`
- `name`, `version`, `secret`, `custom_path`, `draft`, `description`, `title_prefix`, `bibliography` を管理。

- `SlideType`
- `Marp`（`slide.md`）または `Ipe`（`slide.ipe`）。

## 処理フロー（代表）

### `build`

1. 指定されたディレクトリごとに `Project::get_slide` でスライド解決
2. Ipe の場合は `slide.pdf` を `output/*.pdf` へコピー
3. Marp の場合は HTML/PDF コマンドを生成
4. `images/` を `output/<stem>/images` へコピー
5. 非 `draft` スライドのみを対象に並列実行

`stem` は次の優先順で決まります。

- `custom_path`（0個以上）
- `secret` があれば UUID
- なければ `name`

そのため、1スライドから複数出力（別パス）を生成できます。

### `pre-commit`

1. いま存在する `slide.toml` 一覧から「残すべき出力名」を計算
2. `output` 配下の不要ファイル/ディレクトリを削除
3. テンプレートを使って `README.md` と `output/index.html` を再生成

## 設計上の注意点

- `add` の `--secret` は既定で `true`
- 明示しないと UUID ベースの秘密URLが作られる
- 公開スライドを追加したい場合は `--secret false` を指定

- `index` / `bib` は入力ファイルを書き換える
- 実行前に差分確認しやすい運用（Git 管理）を推奨

- `build` は外部コマンド（Marp CLI）依存
- `build.marp_binary` のパス解決に失敗するとビルド失敗
