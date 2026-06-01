方針としては，`slide-flow build` の内部に「画像最適化ステージ」を追加しつつ，単体でも実行できる `slide-flow images optimize` を用意するのがよいです．

つまり，通常利用では次の流れにします．

```txt
slide-flow build slide.md

1. slide.md を読む
2. 参照されている画像を収集する
3. 画像をキャッシュディレクトリへ最適化する
4. 一時的なビルド用 Markdown / assets を作る
5. Marp で HTML / PDF / OGP を生成する
6. 公開用ディレクトリへ配置する
```

ポイントは，元画像を直接書き換えないことです．スライド用画像は，あとから再編集したり，高解像度版を残したりすることが多いので，`assets/` 以下の原本は保持し，`target/slide-flow/assets/` や `.slide-flow/cache/` に最適化済み画像を出す設計が安全です．

ディレクトリは次のようにすると扱いやすいです．

```txt
slides/
├── sources/
│   ├── talk.md
│   └── assets/
│       ├── graph.png
│       ├── diagram.svg
│       └── photo.jpg
├── .slide-flow/
│   └── cache/
│       └── images/
│           ├── <hash>.png
│           ├── <hash>.svg
│           └── <hash>.jpg
└── public/
    └── ...
```

`build` 時には，`talk.md` 内の画像参照を解析して，ビルド用の一時 Markdown を作ります．たとえば元の Markdown が次のような場合です．

```md
![graph](./assets/graph.png)
<img src="./assets/diagram.svg" />
```

内部では，一時ファイル上だけ次のように差し替えます．

```md
![graph](../.slide-flow/cache/images/<hash>.png)
<img src="../.slide-flow/cache/images/<hash>.svg" />
```

これにより，Marp が HTML や PDF を作る時点で最適化済み画像を参照できます．「ビルド前に実行」という要望にはこの形が一番合います．

CLI は次のように分けるとよいです．

```txt
slide-flow build <source>
slide-flow build-all

slide-flow images optimize <source>
slide-flow images optimize-all
slide-flow images check <source>
slide-flow images clean
```

`slide-flow build` はデフォルトで画像最適化を実行します．一方で，デバッグや差分確認のために次も用意します．

```txt
slide-flow build talk.md --no-optimize-images
slide-flow images optimize talk.md --dry-run
slide-flow images optimize talk.md --force
```

`--dry-run` では，「どの画像が対象になるか」「何 byte 減る見込みか」「どのツールが使われるか」だけを表示します．`--force` はキャッシュを無視して再最適化します．

設定ファイルは，最初はこのくらいで十分です．

```toml
[images]
enabled = true
cache_dir = ".slide-flow/cache/images"
mode = "lossless"
strip_metadata = true
fail_on_missing_tool = false

[images.png]
enabled = true
tool = "oxipng"
level = 4

[images.jpeg]
enabled = true
tool = "jpegoptim"
quality = 85

[images.svg]
enabled = true
tool = "svgo"

[images.webp]
enabled = false
```

ただし，`mode = "lossless"` をデフォルトにするのが無難です．スライドでは図，スクリーンショット，数式画像などが多く，勝手に画質を落とすと困ることがあります．JPEG だけ `quality = 85` のような lossy 圧縮を許す場合は，明示的に `mode = "lossy"` を指定する形がよいです．

画像形式ごとの扱いは次の方針がよいです．

```txt
PNG  -> oxipng
JPEG -> jpegoptim
SVG  -> svgo
WebP -> 初期実装ではコピーのみ，後から対応
AVIF -> 初期実装では非対応
GIF  -> 初期実装ではコピーのみ
```

`oxipng` は PNG/APNG の lossless 最適化用の CLI / Rust ライブラリで，macOS / Linux ではパッケージマネージャや Cargo から入れる方針が案内されています．([GitHub][1]) `jpegoptim` は JPEG の最適化・圧縮用ユーティリティです．([GitHub][2]) `svgo` は SVG 最適化用の Node.js ライブラリ兼 CLI で，`npm install -g svgo` などで導入できます．([GitHub][3])

Ubuntu / macOS 両対応を考えると，最初から全 optimizer を Rust に組み込むより，外部コマンドとして呼ぶ設計が素直です．`slide-flow doctor` で依存コマンドの有無を確認できるようにします．

```txt
slide-flow doctor

marp       ok
oxipng     ok
jpegoptim  missing
svgo       ok
```

`fail_on_missing_tool = false` の場合，未インストールの optimizer があっても，その形式の画像はそのままコピーしてビルドを継続します．これは個人用ツールとして使いやすいです．CI では `fail_on_missing_tool = true` にするとよいです．

Rust 側の構成は，たとえば次のように分けます．

```txt
src/
├── main.rs
├── cli.rs
├── config.rs
├── build/
│   ├── mod.rs
│   ├── pipeline.rs
│   └── marp.rs
├── images/
│   ├── mod.rs
│   ├── collect.rs
│   ├── cache.rs
│   ├── optimizer.rs
│   ├── rewrite.rs
│   └── report.rs
└── fs.rs
```

`images::collect` は Markdown / HTML 風の画像参照を集めます．最初は Markdown の `![](...)` と `<img src="...">` だけ対応すればよいです．CSS の `background-image` まで追うのは後回しでよいです．

`images::cache` は入力ファイルのパス，更新時刻，サイズ，optimizer 設定をもとにキャッシュキーを作ります．より堅くするならファイル内容のハッシュを使います．

```txt
cache key = hash(
  absolute source path,
  file content hash,
  optimizer kind,
  optimizer options
)
```

これにより，画像が変わっていない場合は最適化をスキップできます．同じスライドを何度もビルドするときに効きます．

`images::optimizer` は次のような trait にすると拡張しやすいです．

```rust
pub trait ImageOptimizer {
    fn supports(&self, ext: &str) -> bool;
    fn optimize(&self, input: &Path, output: &Path, options: &ImageOptions) -> Result<OptimizeResult>;
}
```

ただし，これはやや抽象化が早い可能性があります．初期実装では `match ext` で `Command` を呼ぶだけでも十分です．将来 WebP / AVIF / 自前 Rust 実装を入れたいなら trait 化，まず動かすなら `match` でよいです．

ビルドパイプラインとしては，次の順序がよいです．

```txt
BuildPipeline
  -> resolve slide metadata
  -> collect image refs
  -> optimize images into cache
  -> generate staged markdown
  -> run marp html
  -> run marp pdf
  -> generate ogp
  -> write redirects
```

ここで重要なのは，OGP 用画像の最適化は別扱いにすることです．スライド内で使う画像は「ビルド前」に最適化しますが，OGP 画像は Marp などで生成された後にできるので，「ビルド後」に最適化します．したがって，画像最適化は厳密には 2 種類あります．

```txt
pre-build image optimization
  スライド本文が参照する画像を最適化する

post-build image optimization
  生成された OGP 画像やサムネイルを最適化する
```

設定上も分けられるようにしておくとよいです．

```toml
[images.pre_build]
enabled = true

[images.post_build]
enabled = true
targets = ["ogp"]
```

最初の MVP は次で十分です．

```txt
1. Markdown の画像参照を収集
2. PNG / JPEG / SVG を対象にする
3. 元画像は変更しない
4. `.slide-flow/cache/images` に最適化済み画像を生成
5. 一時 Markdown の参照先を差し替えて Marp を実行
6. `--no-optimize-images` と `--dry-run` を用意
7. `doctor` で依存コマンドを確認
```

この設計なら，Ubuntu / macOS の差はほぼ「外部コマンドのインストール方法」だけになります．`slide-flow` 本体は Rust で共通にし，optimizer は PATH 上のコマンドとして呼び出すのが一番実装しやすいです．

README には次のように書ける形を目標にするとよいです．

```bash
# Ubuntu
sudo apt install oxipng jpegoptim
npm install -g svgo

# macOS
brew install oxipng jpegoptim
npm install -g svgo
```

ただし，パッケージ名や提供状況は環境で変わる可能性があるので，`slide-flow doctor` で最終確認する運用にしておくのが安全です．

[1]: https://github.com/shssoichiro/oxipng "GitHub - oxipng/oxipng: Multithreaded PNG optimizer written in Rust · GitHub"
[2]: https://github.com/tjko/jpegoptim "GitHub - tjko/jpegoptim: jpegoptim - utility to optimize/compress JPEG files · GitHub"
[3]: https://github.com/svg/svgo "GitHub - svg/svgo: ⚙️ Node.js tool for optimizing SVG files · GitHub"

