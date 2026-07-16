# slide-flow

`slide-flow` is a Rust CLI for managing slide decks authored with Marp markdown or Ipe PDF sources. It creates slide workspaces, manages versions, builds HTML / PDF / OGP artifacts, updates tables of contents, and supports migration from legacy public paths to canonical UUID-backed paths with redirects.

[日本語版](README-ja.md)

## Requirements

- Rust and Cargo
- Marp CLI available as `marp`, or another command configured in `config.toml`
- Ipe is optional, only needed when using `type = "ipe"`

## Installation

```bash
cargo install --git https://github.com/kentakom1213/slide-flow -f
```

## Quick Start

```bash
slide-flow init
slide-flow slide add my-first-slide
slide-flow toc src/my-first-slide
slide-flow build src/my-first-slide
slide-flow project list
```

The generated output is written to `output_dir` from `config.toml`. The default is `output/`.

## Commands

Top-level commands:

```txt
slide-flow init
slide-flow build <DIR>... | --all | --changed
slide-flow prepare [<DIR>... | --all | --changed]
slide-flow toc <DIR>... | --all | --changed
slide-flow bib <DIR>... | --all | --changed
slide-flow slide <COMMAND>
slide-flow project <COMMAND>
slide-flow images <COMMAND>
slide-flow prune <COMMAND>
slide-flow migrate <COMMAND>
```

Slide commands:

```txt
slide-flow slide add <NAME> [--secret | --public] [--draft] [--type <marp|ipe>]
slide-flow slide show <NUMBER|DIR>
slide-flow slide archive <DIR>
```

Project commands:

```txt
slide-flow project list
slide-flow project show
slide-flow project refresh
```

Image and pruning commands:

```txt
slide-flow images optimize <DIR>... | --all | --changed [--dry-run] [--force]
slide-flow images clean
slide-flow prune outputs [--dry-run]
slide-flow prune outputs --apply
```

Migration commands:

```txt
slide-flow migrate plan [DIR]
slide-flow migrate status
slide-flow migrate apply <DIR> --metadata-only
slide-flow migrate apply <DIR> --redirects-only
slide-flow migrate apply <DIR> --artifacts [--concurrent 4]
slide-flow migrate apply <DIR> --remove-legacy-artifacts
```

## Project Layout

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

## Creating Slides

Create a secret Marp slide:

```bash
slide-flow slide add my-first-slide
```

Create a public slide:

```bash
slide-flow slide add public-talk --public
```

Create a draft:

```bash
slide-flow slide add work-in-progress --draft
```

Create an Ipe slide:

```bash
slide-flow slide add figure-talk --type ipe
```

Each slide lives under `src/<name>/` and has a `slide.toml`. Marp slides use `slide.md`; Ipe slides use `slide.ipe` and `slide.pdf`.

## Managing Slides

List slides:

```bash
slide-flow project list
```

Show metadata and published URLs:

```bash
slide-flow slide show 1
slide-flow slide show src/my-first-slide
```

Archive the current version before starting a new revision:

```bash
slide-flow slide archive src/my-first-slide
```

This copies the current slide files into `src/my-first-slide/v<version>/`, increments `version`, and recreates the working slide files.

## Indexing and Bibliography

Add slide numbers and a table of contents to one slide:

```bash
slide-flow toc src/my-first-slide
```

Index all slides:

```bash
slide-flow toc --all
```

Update bibliography entries for a slide:

```bash
slide-flow bib src/my-first-slide
```

## Building

Build one or more slides:

```bash
slide-flow build src/my-first-slide
slide-flow build src/my-first-slide src/another-slide
slide-flow build src/my-first-slide --concurrent 8
slide-flow build --all
slide-flow build --changed
```

For Marp slides, `slide-flow` invokes Marp and builds HTML and PDF artifacts. For Ipe slides, it copies `slide.pdf` into the output directory. Archived versions under `src/<slide>/v*/` are built as versioned PDFs; with `canonical-with-redirects`, archived Marp versions also get versioned HTML and OGP images.

## Preparing Publish Files

Run the standard publish preparation pipeline:

```bash
slide-flow prepare
```

By default, `prepare` targets changed slides and runs project refresh, table of contents updates, bibliography updates, builds, and stale output pruning. Preview the selected targets and planned steps:

```bash
slide-flow prepare --dry-run
```

Recommended hook usage keeps staging outside `slide-flow`:

```bash
slide-flow prepare --no-build
dprint fmt
slide-flow build --changed --concurrent 4
git add docs README.md src
```

Stale output pruning can also be run explicitly. It defaults to dry-run unless `--apply` is passed:

```bash
slide-flow prune outputs --dry-run
slide-flow prune outputs --apply
```

## Path Strategies

`slide-flow` supports two path strategies:

- `legacy`: keeps the historical behavior. All `custom_path` values and the canonical stem are treated as real output locations.
- `canonical-with-redirects`: writes real artifacts under the canonical stem and writes redirect HTML under aliases.

The strategy is resolved in this order:

```txt
slide.toml
> config.toml
> default legacy
```

Project default:

```toml
[build]
theme_dir = ".marp/themes"
marp_binary = "marp"
path_strategy = "legacy"
```

Slide override:

```toml
name = "my-first-slide"
version = 1
secret = "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
custom_path = ["my-first-slide"]
path_strategy = "canonical-with-redirects"
```

### Legacy Output

For a slide with `custom_path = ["my-first-slide"]` and `secret = "<uuid>"`, legacy builds write real artifacts for both stems:

```txt
output/my-first-slide/index.html
output/<uuid>/index.html
output/my-first-slide.pdf
output/<uuid>.pdf
output/my-first-slide_v1.pdf
output/<uuid>_v1.pdf
```

### Canonical With Redirects Output

With `canonical-with-redirects`, real artifacts are written under the canonical stem. If `secret` exists, the canonical stem is the UUID. Otherwise it is `name`.

```txt
output/<uuid>/index.html
output/<uuid>/ogp.png
output/<uuid>/v1/index.html
output/<uuid>/v1/ogp.png
output/<uuid>_v1.pdf
output/<uuid>/pdf/index.html
output/<uuid>/pdf/v1/index.html
```

Aliases become redirect HTML:

```txt
output/my-first-slide/index.html
output/my-first-slide/v1/index.html
output/my-first-slide/pdf/index.html
output/my-first-slide/pdf/v1/index.html
```

The public README and `output/index.html` prefer alias URLs. Redirect HTML includes canonical links, Open Graph metadata, Twitter Card metadata, and a JavaScript redirect.

PDF URLs under both canonical and alias paths are HTML redirect pages. This lets social crawlers read OGP metadata before users are redirected to the real PDF.

## Migration

Use migration commands when moving existing slides from `legacy` to `canonical-with-redirects`.

Preview planned changes:

```bash
slide-flow migrate plan
slide-flow migrate plan src/my-first-slide
```

Check current artifact status:

```bash
slide-flow migrate status
```

Update only `slide.toml`:

```bash
slide-flow migrate apply src/my-first-slide --metadata-only
```

Generate only alias redirect HTML:

```bash
slide-flow migrate apply src/my-first-slide --redirects-only
```

Build canonical artifacts and redirects:

```bash
slide-flow migrate apply src/my-first-slide --artifacts
```

Remove legacy alias artifacts:

```bash
slide-flow migrate apply src/my-first-slide --remove-legacy-artifacts
```

`--remove-legacy-artifacts` is intentionally explicit. It removes alias-side legacy PDFs and alias-side copied images, while keeping alias redirect directories.

## Configuration

Example `config.toml`:

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

Example `slide.toml`:

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

`slide-flow` is licensed under the MIT License. See [LICENSE](LICENSE) for details.
