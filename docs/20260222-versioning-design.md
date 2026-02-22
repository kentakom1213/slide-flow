# Versioning 設計（提案）

## 背景

現状の `slide-flow` では `slide.toml` の `version` は保持されるだけで、ビルドや配信パスには反映されません。  
この設計では、次を実現します。

- `version` 更新時に過去版を `src/<slide>/v<version>/` へ退避
- HTML は `/<stem>_v<version>` で配信
- PDF は最新版を `/<stem>.pdf`、版指定を `/<stem>_v<version>.pdf` で配信

## 要件

ここで `stem` は `custom_path` の各要素 + `secret`（あれば）または `name` を指す。

- 最新版の編集対象は `src/<slide>/` 直下
- 過去版は `src/<slide>/v<version>/` に保持
- `output` 配下の成果物は次の命名規則に統一
- HTML: `output/<stem>_v<version>/index.html`（常に最新のみ）
- PDF: `output/<stem>.pdf`（最新版エイリアス） + `output/<stem>_v<version>.pdf`（複数版を共存）
  - 最新版のみ `output/<slide>.pdf` とする．

## 制約と設計方針

手動で `slide.toml` の `version` だけを書き換えた場合、旧版コンテンツの完全復元はできません。  
そのため、版上げは CLI で明示的に実行する方式にします。

- 追加コマンド案: `slide-flow version bump --dir src/<slide>`
- このコマンドが「退避 + version 更新」を原子的に実行

この方式により、`version` 変更時に必ず過去版が保存される状態を担保します。

## データ配置

### 最新版

```text
src/<slide>/
├── images/
├── slide.md | slide.ipe
└── slide.toml   # version = N
```

### 過去版

```text
src/<slide>/v<N-1>/
├── images/
├── slide.md | slide.ipe
└── slide.toml   # version = N-1
```

## バージョン更新フロー

対象: `src/<slide>/`（現在 version = N）

1. `src/<slide>/vN/` が未存在であることを確認（存在時はエラー）
2. `slide.md` / `slide.ipe` / `images/` / `slide.toml` を `vN/` に移動
3. `src/<slide>/` 直下に新しい作業用ファイルを再作成
4. `slide.toml` の `version` を `N+1` に更新
5. 必要に応じて `template.slide` から新規 `slide.md` を初期化

注意:
- 「移動」としているが、運用上はコピー + 検証 + 元削除の安全手順を推奨
- 失敗時ロールバック方針（後述）を実装する

## build 配信仕様

### 入力ソース

- 最新版: `src/<slide>/slide.md`（または `slide.ipe`）
- 過去版: `src/<slide>/v*/slide.md`（または `slide.ipe`）

### 出力

- 最新版 HTML: `output/<stem>_v<current>/index.html`
- 最新版 PDF: `output/<stem>.pdf` と `output/<stem>_v<current>.pdf`
- 過去版 PDF: `output/<stem>_v<past>.pdf`

方針:
- HTML は最新のみを生成
- PDF は全版を版付き名で生成（履歴保持）

## pre-commit / キャッシュ削除仕様

`remove_cache` は次を保持対象にする。

- `output/<slide>/`（最新版 HTML）
- `output/<slide>_v<k>.pdf`（`src/<slide>/` と `src/<slide>/v*/` に存在する全 version）

これにより、古いが有効な PDF を誤削除しない。

## README / index の表示方針

- スライド一覧は `src/<slide>/slide.toml`（最新版）を基準に表示
- Slide リンクは従来どおり `/<slide>`
- PDF リンクは `/<slide>_v<current>.pdf` を表示
- 過去版 PDF 一覧は初期スコープ外（必要なら将来拡張）

## 既存モジュールへの変更点（設計）

- `src/parser.rs`
- `Version` サブコマンド追加（`bump`）

- `src/subcommand/version.rs`（新規）
- 退避ディレクトリ作成
- ファイル移動/再作成
- `slide.toml` version 更新

- `src/project.rs`
- `src/<slide>/v*/` を列挙する API 追加

- `src/subcommand/build.rs`
- 出力命名を `<slide>_v<version>.pdf` へ変更
- 過去版 PDF 生成を追加
- HTML は最新版のみ生成

- `src/subcommand/pre_commit.rs`
- `remove_cache` の保持判定を version-aware に変更
  - 最新版エイリアス `output/<stem>.pdf` も保持対象にする

- `templates/readme.md`
- PDF リンクの stem を `<slide>_v<version>` へ変更

## 失敗時の扱い

- `bump` は次を満たしたときのみ成功扱い
- `vN/` への保存完了
- 新 `slide.toml`（`version = N+1`）保存完了
- 作業ディレクトリ再初期化完了

- 途中失敗時
- 元ディレクトリを壊さない（先にコピーして検証）
- 不完全な `vN/` を検出可能な状態にする（ログ + エラー）

## テスト観点

- `version bump`
- `vN/` 作成、`version` 更新、既存 `vN/` 衝突時エラー

- `build`
- 最新 HTML が `output/<slide>/index.html`
- 最新/過去 PDF が `output/<slide>_v<version>.pdf`

- `pre-commit`
- 有効な版付き PDF を削除しない
- 無効な成果物のみ削除

## 段階導入案

1. `version bump` 実装（退避保証）
2. PDF 命名を `*_v<version>.pdf` へ変更
3. 過去版 PDF ビルド対応
4. `pre-commit` の保持ロジック更新
5. `README`/テンプレートのリンク更新

この順で進めると、破壊的変更を最小化しながら移行できます。

## 実装仕様（bump 方針）

### 1. CLI 仕様

新規サブコマンドを追加する。

```bash
slide-flow version bump --dir src/<slide>
```

想定構文（`clap`）:

- `Version { action: VersionAction }`
- `VersionAction::Bump { dir: PathBuf }`

補足:
- `dir` は `src/<slide>` を受け取る
- `v<version>` ディレクトリを直接受け取った場合はエラーにする

### 2. 処理責務の分割

- `src/subcommand/version.rs`（新規）
- `pub fn bump(project: &Project, dir: PathBuf) -> anyhow::Result<()>`
- 退避ディレクトリ作成、ファイル退避、version 更新、作業ディレクトリ再初期化

- `src/parser.rs`
- `SubCommands` に `Version` を追加

- `src/subcommand/mod.rs`
- `pub mod version;` を追加

- `src/main.rs`
- `Version` 分岐を追加し `version::bump` を呼ぶ

### 3. bump アルゴリズム

前提: 対象 `src/<slide>/slide.toml` の `version = N`

1. 対象スライドの解決
- `project.get_slide(&dir)` で `Slide` を取得
- `slide.type_` に応じて対象ファイルを決定 (`slide.md` or `slide.ipe`)

2. 退避先確定
- `archive_dir = <slide_dir>/vN`
- `archive_dir` が存在する場合は即エラー

3. 退避対象の収集
- 必須: `slide.toml`
- 条件付き: `slide.md` または `slide.ipe`
- 条件付き: `images/`（存在時）

4. 退避実行（安全順序）
- `archive_dir` 作成
- 対象を `archive_dir` にコピー
- コピー後に存在検証
- 検証成功後、元ファイルを削除

5. 新版作業ディレクトリの再作成
- `images/` を再作成し `.gitkeep` を配置
- スライド本体を `project.conf.template.slide` から再作成（Marp）
- Ipe の場合は空の `slide.ipe` を作るか、未作成として明示エラーにするかを実装時に統一

6. `slide.toml` 更新
- 退避した `slide.toml` を基に `version = N + 1` に更新
- その他フィールド（`name`, `secret`, `draft`, `custom_path` など）は維持
- 更新済み `slide.toml` を `src/<slide>/slide.toml` に保存

7. 完了ログ
- `archived: src/<slide>/vN`
- `bumped: N -> N+1`

### 4. エラー・ロールバック方針

- `archive_dir` 作成前に失敗した場合は何も変更しない
- コピー検証失敗時は元を削除しない
- 元削除途中で失敗した場合は処理停止し、復旧手順をログで案内
- 実装上は「copy -> verify -> remove」の順を厳守し、`rename` のみには依存しない

### 5. build / pre-commit との接続

`bump` 実装後、次の段階で既存処理を version-aware 化する。

- `build`
- `src/<slide>/` を最新版として HTML 出力
- `src/<slide>/` と `src/<slide>/v*/` を対象に PDF 出力名を決定

- `pre-commit remove_cache`
- 保持対象に `output/<slide>_v<version>.pdf` 群を含める

- `templates/readme.md`
- 最新版PDFリンクを version ベースの命名に追従

### 6. 実装タスク分解

1. `parser/main/subcommand mod` に `Version::Bump` の配線追加
2. `src/subcommand/version.rs` を新規作成
3. `bump` の正常系実装（退避 + 再初期化 + version更新）
4. 失敗系のガード実装（重複 `vN`、欠損ファイル、I/Oエラー）
5. ユニットテスト追加
6. 既存 docs (`usage` / `architecture`) への反映

### 7. テストケース（bump）

- 正常系
- `version=1` で `bump` 実行後、`v1/` が作成される
- 新 `slide.toml` が `version=2`
- 旧 `slide.md`/`images` が `v1/` 側に存在
- 新 `src/<slide>/images/.gitkeep` が存在

- 異常系
- `vN/` 既存時にエラー
- `slide.toml` 不在時にエラー
- `slide.md` / `slide.ipe` 不在時にエラー
- コピー検証失敗時に元データが残る

### 8. 非互換変更の扱い

- 既存ユーザーに対しては `version` を手動編集しても過去版退避されない
- 過去版保存を保証する正式手順を `version bump` に一本化
- README と usage に「版上げは `version bump` を使う」ことを明記
