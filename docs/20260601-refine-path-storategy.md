# slide-flow path strategy redesign

## 目的

このドキュメントは，`slide-flow` の出力パス設計，ビルドパイプライン，CLI コマンド，および旧形式からの migration を再設計するための実装方針をまとめるものです．

主な目的は次の通りです．

- 既存の `slides` リポジトリに存在する公開 URL を壊さない．
- 新規スライドでは，HTML / PDF / OGP 画像の重複ビルドを避ける．
- 外部に見せる URL は人間が読みやすい alias に寄せ，実体は UUID に集約する．
- CLI コマンドを整理し，責務を明確にする．
- 旧形式から新形式へ段階的に移行できる migration コマンドを提供する．

## 背景

現在の `slide-flow` では，`custom_path` と `secret` / `name` の両方を出力 stem として扱っている．

例えば，次のような `slide.toml` を考える．

```toml
name = "202606_new_slide"
version = 2
secret = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
custom_path = ["202606_new_slide"]
draft = false
```

現状では，`custom_path` 側と `secret` 側の両方に HTML / PDF 実体が生成される．

```txt
docs/
├── 202606_new_slide/
│   ├── index.html
│   └── images/
│
├── xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx/
│   ├── index.html
│   └── images/
│
├── 202606_new_slide_v2.pdf
├── xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx_v2.pdf
├── 202606_new_slide.pdf
└── xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx.pdf
```

この構成には次の問題がある．

- 同一内容の HTML を複数回 `marp` でビルドしている．
- 同一内容の PDF を複数回 `marp` でビルドしている．
- 画像ディレクトリも複数箇所にコピーされる．
- 同じスライド実体が複数 URL に存在し，canonical URL が曖昧になる．
- `docs/` の容量が増えやすい．

## 新しい URL 設計

新方式では，`secret` を canonical stem，`custom_path` を alias として扱う．

`secret` が存在しない場合は，`name` を canonical stem とする．

```txt
canonical_stem:
  secret があれば secret
  なければ name

alias_stems:
  custom_path の各要素
```

外部に見せる URL は alias 側に寄せる．ただし，実体は canonical 側に置く．

HTML は次のように redirect する．

```txt
/<alias>/
  -> /<uuid>/

/<alias>/v1/
  -> /<uuid>/v1/

/<alias>/v2/
  -> /<uuid>/v2/
```

PDF は次のように redirect する．

```txt
/<alias>/pdf/
  -> /<uuid>_v<latest>.pdf

/<alias>/pdf/v1/
  -> /<uuid>_v1.pdf

/<alias>/pdf/v2/
  -> /<uuid>_v2.pdf
```

ここで，`/<alias>/pdf/` や `/<alias>/pdf/v1/` は PDF ファイルそのものではなく，PDF 実体へ遷移する redirect HTML である．

## 新方式の `docs/` 構成

```txt
docs/
├── <uuid>/
│   ├── index.html              # latest HTML 実体
│   ├── ogp.png                 # latest OGP 画像
│   ├── images/
│   ├── v1/
│   │   ├── index.html          # v1 HTML 実体
│   │   └── ogp.png             # v1 OGP 画像
│   └── v2/
│       ├── index.html          # v2 HTML 実体
│       └── ogp.png             # v2 OGP 画像
│
├── <uuid>_v1.pdf
├── <uuid>_v2.pdf
│
└── <alias>/
    ├── index.html              # /<uuid>/ へ redirect
    ├── v1/
    │   └── index.html          # /<uuid>/v1/ へ redirect
    ├── v2/
    │   └── index.html          # /<uuid>/v2/ へ redirect
    └── pdf/
        ├── index.html          # /<uuid>_v2.pdf へ redirect
        ├── v1/
        │   └── index.html      # /<uuid>_v1.pdf へ redirect
        └── v2/
            └── index.html      # /<uuid>_v2.pdf へ redirect
```

`alias` 側はすべて redirect HTML とする．PDF alias も `.pdf` コピーではなく，`/<alias>/pdf/` 形式の redirect HTML として生成する．

## path strategy

既存プロジェクトを壊さないため，strategy はスライド単位で切り替えられるようにする．

```toml
# config.toml
[build]
path_strategy = "legacy"
```

```toml
# src/202606_new_slide/slide.toml
path_strategy = "canonical-with-redirects"
```

設定の優先順位は次の通りとする．

```txt
slide.toml
  > config.toml
  > slide-flow default
```

暗黙のデフォルトは `legacy` とする．これにより，既存プロジェクトの `config.toml` に新項目が存在しない場合でも，従来挙動を維持できる．

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathStrategy {
    Legacy,
    CanonicalWithRedirects,
}
```

## `legacy` strategy

`legacy` は従来挙動を維持するための strategy である．

`custom_path` と `secret` / `name` の全てを実体出力先として扱う．

```txt
docs/<alias>/index.html
docs/<uuid>/index.html

docs/<alias>_vN.pdf
docs/<uuid>_vN.pdf

docs/<alias>.pdf
docs/<uuid>.pdf
```

既存の `slides` リポジトリの URL 互換性を保つために残す．

## `canonical-with-redirects` strategy

新規スライド向けの推奨 strategy である．

- HTML 実体は canonical 側にだけ生成する．
- PDF 実体は canonical 側にだけ生成する．
- OGP 画像は canonical 側にだけ生成する．
- alias 側は HTML redirect のみ生成する．
- PDF alias は `.pdf` コピーではなく，`/<alias>/pdf/` の redirect HTML とする．

```txt
canonical 実体:
  /<uuid>/
  /<uuid>/v1/
  /<uuid>/v2/
  /<uuid>_v1.pdf
  /<uuid>_v2.pdf

alias redirect:
  /<alias>/
  /<alias>/v1/
  /<alias>/v2/
  /<alias>/pdf/
  /<alias>/pdf/v1/
  /<alias>/pdf/v2/
```

## versioned HTML

新方式では，`/<alias>/vN/ -> /<uuid>/vN/` を提供するため，archived version に対しても HTML を生成する．

```txt
src/<slide>/v1/slide.md
  -> docs/<uuid>/v1/index.html
  -> docs/<uuid>/v1/ogp.png
  -> docs/<uuid>_v1.pdf
```

latest version については，次を生成する．

```txt
src/<slide>/slide.md
  -> docs/<uuid>/index.html
  -> docs/<uuid>/ogp.png
  -> docs/<uuid>_v<latest>.pdf
```

また，latest version に対しても固定 URL を持たせるため，`/<uuid>/v<latest>/` も生成することを推奨する．

```txt
/<uuid>/
  latest への入口

/<uuid>/v2/
  v2 固定の入口
```

## redirect HTML

redirect HTML は，絶対 URL で遷移先を指定する．また，OGP / Twitter Card も設定する．

```html
<!doctype html>
<html lang="ja">
  <head>
    <meta charset="utf-8" />
    <title>202606_new_slide</title>

    <link rel="canonical" href="https://kentakom1213.github.io/slides/<uuid>/" />
    <meta http-equiv="refresh" content="0; url=https://kentakom1213.github.io/slides/<uuid>/" />

    <meta name="description" content="スライドの説明文" />

    <meta property="og:type" content="website" />
    <meta property="og:title" content="202606_new_slide" />
    <meta property="og:description" content="スライドの説明文" />
    <meta property="og:url" content="https://kentakom1213.github.io/slides/<uuid>/" />
    <meta property="og:site_name" content="slides" />
    <meta property="og:image" content="https://kentakom1213.github.io/slides/<uuid>/ogp.png" />

    <meta name="twitter:card" content="summary_large_image" />
    <meta name="twitter:title" content="202606_new_slide" />
    <meta name="twitter:description" content="スライドの説明文" />
    <meta name="twitter:image" content="https://kentakom1213.github.io/slides/<uuid>/ogp.png" />

    <script>
      location.replace("https://kentakom1213.github.io/slides/<uuid>/");
    </script>
  </head>
  <body>
    <p>
      Redirecting to
      <a href="https://kentakom1213.github.io/slides/<uuid>/">
        https://kentakom1213.github.io/slides/<uuid>/
      </a>
    </p>
  </body>
</html>
```

HTML version redirect では，`og:image` は `<base_url>/<uuid>/vN/ogp.png` を指す．PDF latest redirect では，`og:image` は `<base_url>/<uuid>/ogp.png` を指す．PDF version redirect では，`og:image` は `<base_url>/<uuid>/vN/ogp.png` を指す．

## OGP 画像生成

`marp` CLI でスライドの先頭ページを画像として出力し，それを `og:image` に利用する．

```txt
latest:
  docs/<uuid>/ogp.png

versioned:
  docs/<uuid>/vN/ogp.png
```

`marp` コマンドのイメージは次の通り．

```sh
marp src/202606_new_slide/slide.md \
  --theme-set .marp/themes \
  --html true \
  --image png \
  -o docs/<uuid>/ogp.png
```

実装上は `BuildCommand` に `OGPImage` を追加する．

```rust
pub enum BuildCommand {
    PDF {
        dir: PathBuf,
        command: Command,
        conf: SlideConf,
        temp_input: Option<PathBuf>,
    },
    HTML {
        dir: PathBuf,
        command: Command,
        conf: SlideConf,
        temp_input: Option<PathBuf>,
    },
    OGPImage {
        dir: PathBuf,
        command: Command,
        conf: SlideConf,
        temp_input: Option<PathBuf>,
    },
}
```

## 内部モデルの整理

現在の `make_file_stems` は，公開 URL と実体出力先を同時に扱っている．これを分離する．

```rust
pub fn canonical_stem(slide: &Slide) -> String {
    slide
        .conf
        .secret
        .clone()
        .unwrap_or_else(|| slide.conf.name.clone())
}

pub fn alias_stems(slide: &Slide) -> Vec<String> {
    slide.conf.custom_path.clone().unwrap_or_default()
}

pub fn legacy_file_stems(slide: &Slide) -> Vec<String> {
    let mut res = slide.conf.custom_path.clone().unwrap_or_default();

    if let Some(secret) = &slide.conf.secret {
        res.push(secret.clone());
    } else {
        res.push(slide.conf.name.clone());
    }

    res
}
```

さらに，`Slide + Project` から `PublishPlan` を作り，その後に `BuildPlan` を作る構成にする．

```txt
Slide + Project
  -> PublishPlan
  -> BuildPlan
  -> execute
```

## CLI コマンドの整理

CLI は次のように整理する．

```txt
slide-flow init

slide-flow slide add
slide-flow slide list
slide-flow slide show
slide-flow slide archive

slide-flow build
slide-flow publish
slide-flow clean

slide-flow migrate plan
slide-flow migrate status
slide-flow migrate apply
```

既存コマンドはすぐに消さず，互換 alias として残す．

```txt
slide-flow add
  -> slide-flow slide add

slide-flow slides list
  -> slide-flow slide list

slide-flow slide <selector>
  -> slide-flow slide show <selector>

slide-flow version bump
  -> slide-flow slide archive

slide-flow pre-commit
  -> slide-flow publish --all --clean
```

互換コマンドを使った場合は warning を出す．

## migration コマンド

migration は専用サブコマンドとして提供する．

```txt
slide-flow migrate plan
slide-flow migrate status
slide-flow migrate apply
```

migration は次の 2 種類に分ける．

```txt
metadata migration:
  slide.toml に path_strategy などを追記する

artifact migration:
  docs/ 以下の生成物を整理する
```

初期実装では，metadata migration と redirect 生成までを優先する．旧 artifact の削除は後回しにする．

### `migrate plan`

```sh
slide-flow migrate plan
slide-flow migrate plan src/202606_new_slide
```

変更予定を表示するだけで，ファイルは変更しない．

### `migrate status`

```sh
slide-flow migrate status
```

各スライドの strategy と生成物の状態を表示する．

### `migrate apply --metadata-only`

```sh
slide-flow migrate apply src/202606_new_slide --metadata-only
```

`slide.toml` だけを変更する．既存の `docs/` は触らない．

### `migrate apply --redirects-only`

```sh
slide-flow migrate apply src/202606_new_slide --redirects-only
```

alias redirect HTML だけを生成する．

### `migrate apply --artifacts`

```sh
slide-flow migrate apply src/202606_new_slide --artifacts
```

canonical 実体の生成や redirect 生成まで行う．旧 artifact は削除しない．

### `migrate apply --remove-legacy-artifacts`

```sh
slide-flow migrate apply src/202606_new_slide --remove-legacy-artifacts
```

旧 artifact を削除する．これは危険なので，明示 opt-in にする．特に，旧 `.pdf` URL を維持したい場合は削除しない．

## README / index のリンク方針

新方式では，README と `docs/index.html` には alias URL を表示する．

```txt
Slide:
  /<alias>/

PDF:
  /<alias>/pdf/

Versions:
  /<alias>/v1/
  /<alias>/pdf/v1/
```

実体 URL である `/<uuid>/` は原則として外に見せない．ただし，redirect HTML の `canonical` は `/<uuid>/` を指す．

## 導入方針

既存の `slides` リポジトリでは，`config.toml` は当面 `legacy` のままにする．

```toml
[build]
path_strategy = "legacy"
```

既存スライドの `slide.toml` は変更しない．新規スライドだけ，`path_strategy = "canonical-with-redirects"` を指定する．

## 実装ステップ

### PR 1: CLI コマンド整理

- `slide-flow slide add` を追加する．
- `slide-flow slide list` を追加する．
- `slide-flow slide show` を追加する．
- `slide-flow slide archive` を追加する．
- 既存コマンドは互換 alias として残す．
- deprecated warning を出す．

### PR 2: PathStrategy / PublishPlan 導入

- `PathStrategy` を追加する．
- `BuildConf` に project default を追加する．
- `SlideConf` に slide-local override を追加する．
- `canonical_stem` / `alias_stems` / `legacy_file_stems` を追加する．
- legacy 挙動を `PublishPlan` 経由に置き換える．
- 外部挙動は変えない．

### PR 3: canonical-with-redirects 実装

- 新方式の canonical HTML / PDF 出力を追加する．
- archived version の HTML 生成を追加する．
- alias HTML redirect を生成する．
- PDF redirect を生成する．
- README / index の URL 生成を strategy 対応にする．

### PR 4: OGP 画像生成

- `marp --image png` による OGP 画像生成を追加する．
- latest / versioned HTML に対応する `ogp.png` を生成する．
- redirect HTML に `og:image` / `twitter:image` を入れる．

### PR 5: migration 基本機能

- `slide-flow migrate plan` を追加する．
- `slide-flow migrate status` を追加する．
- `slide-flow migrate apply --metadata-only` を追加する．
- `slide-flow migrate apply --redirects-only` を追加する．

### PR 6: artifact migration

- `slide-flow migrate apply --artifacts` を追加する．
- 旧 artifact 削除は明示 opt-in にする．
- 旧 PDF URL 削除はさらに明示 opt-in にする．

## 注意点

`/<alias>/pdf/` はブラウザ上では PDF に遷移するが，URL 自体は HTML redirect である．そのため，`.pdf` で終わる URL を期待する一部のツールとは相性がよくない．ただし，GitHub Pages 上で容量を抑えつつ redirect を実現するには妥当な設計である．

`custom_path` は URL path segment として安全な文字列に制限するのが望ましい．少なくとも，`/`，`"`，`<`，`>`，空白を含む値は避ける．

redirect HTML 生成時には，`title` や `description` を HTML escape する．`canonical_stem` や `alias` も URL として安全な値に制限する．

## 結論

`slide-flow` は，今後次の責務分離にする．

```txt
metadata 管理:
  slide.toml / config.toml / version 管理

build:
  Marp による HTML / PDF / OGP 画像の実体生成

publish:
  README / index / redirect HTML 生成

migration:
  旧形式から新形式への段階的移行
```

既存スライドは `legacy` のまま維持し，新規スライドのみ `canonical-with-redirects` にする．これにより，既存 URL を壊さず，将来のスライドについては重複ビルドと容量増加を抑えられる．

補足です．こちらからリポジトリへ直接作成しようとしましたが，GitHub 連携側で別リポジトリの作成 API に解決されてしまい，`Resource not accessible by integration` で保存できませんでした．上の内容をそのまま `docs/20260601-path-strategy-redesign.md` に置けば使えます．
