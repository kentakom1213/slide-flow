# slide-flow

`slide-flow` は，Marp markdown または Ipe PDF で作るスライドを管理する Rust 製 CLI です，スライド作業環境の作成，バージョン管理，HTML / PDF / OGP 画像のビルド，インデックス生成，旧 URL から canonical UUID ベース URL への migration を扱います，

[English](README.md)

## 必要なもの

- Rust と Cargo
- `marp` として実行できる Marp CLI，または `config.toml` で指定した別コマンド
- Ipe は任意です，`type = "ipe"` のスライドでのみ必要です，

## インストール

```bash
cargo install --git https://github.com/kentakom1213/slide-flow -f
```

## クイックスタート

```bash
slide-flow init
slide-flow slide add my-first-slide
slide-flow slide index --dir src/my-first-slide
slide-flow build src/my-first-slide
slide-flow slide list
```

生成物は `config.toml` の `output_dir` に出力されます，デフォルトは `output/` です，

## コマンド

トップレベルのコマンドです，

```txt
slide-flow init
slide-flow build <DIR>...
slide-flow slide <COMMAND>
slide-flow migrate <COMMAND>
```

スライド操作です，

```txt
slide-flow slide add <NAME> [--secret <true|false>] [--draft <true|false>] [--type <marp|ipe>]
slide-flow slide list
slide-flow slide show <NUMBER|DIR>
slide-flow slide archive <DIR>
slide-flow slide index [--dir <DIR>] [--quiet]
slide-flow slide bib <DIR>
```

移行操作です，

```txt
slide-flow migrate plan [DIR]
slide-flow migrate status
slide-flow migrate apply <DIR> --metadata-only
slide-flow migrate apply <DIR> --redirects-only
slide-flow migrate apply <DIR> --artifacts [--concurrent 4]
slide-flow migrate apply <DIR> --remove-legacy-artifacts
```

## プロジェクト構成

```txt
.
├── .marp/
│   └── themes/
├── src/
│   └── my-first-slide/
│       ├── images/
│       ├── slide.md
│       ├── slide.toml
│       └── v1/
│           ├── images/
│           ├── slide.md
│           └── slide.toml
├── output/
│   ├── index.html
│   ├── <stem>/
│   │   └── index.html
│   └── <stem>_v1.pdf
└── config.toml
```

## スライド作成

Marp スライドを作ります，

```bash
slide-flow slide add my-first-slide
```

draft を作ります，

```bash
slide-flow slide add work-in-progress --draft true
```

Ipe スライドを作ります，

```bash
slide-flow slide add figure-talk --type ipe
```

各スライドは `src/<name>/` に作られ，`slide.toml` を持ちます，Marp スライドは `slide.md`，Ipe スライドは `slide.ipe` と `slide.pdf` を使います，

## スライド管理

一覧を表示します，

```bash
slide-flow slide list
```

メタデータと公開 URL を表示します，

```bash
slide-flow slide show 1
slide-flow slide show src/my-first-slide
```

現在の版を保存して，新しい revision を始めます，

```bash
slide-flow slide archive src/my-first-slide
```

この操作は現在のファイルを `src/my-first-slide/v<version>/` にコピーし，`version` を増やして，作業用ファイルを再作成します，

## インデックスと文献情報

1 つのスライドにページ番号と目次を入れます，

```bash
slide-flow slide index --dir src/my-first-slide
```

全スライドを処理します，

```bash
slide-flow slide index
```

スライドの文献情報を更新します，

```bash
slide-flow slide bib src/my-first-slide
```

## ビルド

1 つ以上のスライドをビルドします，

```bash
slide-flow build src/my-first-slide
slide-flow build src/my-first-slide src/another-slide
slide-flow build src/my-first-slide --concurrent 8
```

Marp スライドでは Marp CLI を呼び出し，HTML と PDF を生成します，Ipe スライドでは `slide.pdf` を出力先へコピーします，`src/<slide>/v*/` の archived version は versioned PDF としてビルドされます，`canonical-with-redirects` では archived Marp version の versioned HTML と OGP 画像も生成されます，

## Path Strategy

`slide-flow` は 2 つの path strategy を持ちます，

- `legacy`: 従来挙動です，`custom_path` と canonical stem のすべてを実体出力先として扱います，
- `canonical-with-redirects`: 実体は canonical stem 側に出力し，alias 側には redirect HTML を出力します，

strategy は次の順で決まります，

```txt
slide.toml
> config.toml
> default legacy
```

project default の例です，

```toml
[build]
theme_dir = ".marp/themes"
marp_binary = "marp"
path_strategy = "legacy"
```

slide override の例です，

```toml
name = "my-first-slide"
version = 1
secret = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
custom_path = ["my-first-slide"]
path_strategy = "canonical-with-redirects"
```

### Legacy Output

`custom_path = ["my-first-slide"]` と `secret = "<uuid>"` を持つスライドでは，legacy は両方の stem に実体を出力します，

```txt
output/my-first-slide/index.html
output/<uuid>/index.html
output/my-first-slide.pdf
output/<uuid>.pdf
output/my-first-slide_v1.pdf
output/<uuid>_v1.pdf
```

### Canonical With Redirects Output

`canonical-with-redirects` では，実体は canonical stem 側に出力されます，`secret` がある場合，canonical stem は UUID です，ない場合は `name` です，

```txt
output/<uuid>/index.html
output/<uuid>/ogp.png
output/<uuid>/v1/index.html
output/<uuid>/v1/ogp.png
output/<uuid>_v1.pdf
output/<uuid>/pdf/index.html
output/<uuid>/pdf/v1/index.html
```

alias 側は redirect HTML になります，

```txt
output/my-first-slide/index.html
output/my-first-slide/v1/index.html
output/my-first-slide/pdf/index.html
output/my-first-slide/pdf/v1/index.html
```

公開用 README と `output/index.html` では alias URL が優先されます，redirect HTML には canonical link，Open Graph metadata，Twitter Card metadata，JavaScript redirect が入ります，

canonical と alias の PDF URL はどちらも HTML redirect page です，これにより，利用者が PDF に遷移する前に social crawler が OGP metadata を読めます，

## Migration

既存スライドを `legacy` から `canonical-with-redirects` に移すときに使います，

変更予定を表示します，

```bash
slide-flow migrate plan
slide-flow migrate plan src/my-first-slide
```

生成物の状態を確認します，

```bash
slide-flow migrate status
```

`slide.toml` だけを更新します，

```bash
slide-flow migrate apply src/my-first-slide --metadata-only
```

alias redirect HTML だけを生成します，

```bash
slide-flow migrate apply src/my-first-slide --redirects-only
```

canonical artifact と redirects を生成します，

```bash
slide-flow migrate apply src/my-first-slide --artifacts
```

legacy alias artifact を削除します，

```bash
slide-flow migrate apply src/my-first-slide --remove-legacy-artifacts
```

`--remove-legacy-artifacts` は明示的に指定する必要があります，alias 側の旧 PDF とコピーされた `images/` を削除しますが，redirect 用の alias directory は残します，

## 設定

`config.toml` の例です，

```toml
name = "My Slides"
author = "Kenta Komoto"
base_url = "https://example.com/slides/"
output_dir = "output"

[template]
slide = "# New Slide\n"
index = ""
suffix = ""

[build]
theme_dir = ".marp/themes"
marp_binary = "marp"
path_strategy = "legacy"
```

`slide.toml` の例です，

```toml
name = "my-first-slide"
version = 1
secret = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
custom_path = ["my-first-slide"]
draft = false
description = "An introduction to the project."
title_prefix = "#"
type = "marp"
path_strategy = "canonical-with-redirects"
```

## License

`slide-flow` は MIT License です，詳細は [LICENSE](LICENSE) を参照してください，
