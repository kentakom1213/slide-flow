# slide-flow

`slide-flow` is a powerful and efficient command-line interface (CLI) tool written in Rust, designed to streamline the creation, management, and publishing of presentations authored in [Marp](https://marp.app/) markdown format. It automates common tasks such as project initialization, slide creation, indexing, and building, making your slide workflow more productive.

## Features

- **Project Initialization**: Quickly set up a new `slide-flow` project with essential directories and a default configuration file.
- **Slide Management**: Easily add new slides to your project.
- **Automated Indexing**: Generate an `index.html` and `README.md` that list your slides, with support for public and secret slides.
- **Efficient Building**: Compile your Marp markdown slides into HTML and PDF formats, with concurrent build capabilities.
- **Cache Management**: Clean up generated build artifacts to maintain a tidy project.
- **Customizable**: Configure project-wide settings, slide templates, and build options via `config.toml`.

## Installation

To install `slide-flow`, ensure you have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed. Then, run the following command:

```bash
cargo install slide-flow
```

## Usage

Navigate to your desired project directory and use the `slide-flow` commands.

### Initialize a New Project

To start a new `slide-flow` project in the current directory:

```bash
slide-flow init
```

This command will create the following basic directory structure:

```
.
├── .marp/
│   └── themes/
│       └── .gitkeep
├── src/
│   └── .gitkeep
└── config.toml
```

The `config.toml` file will be pre-populated with default settings, which you can customize:

```toml
# config.toml
name = "my-slide-project"
author = "Your Name"
base_url = "[https://example.com/](https://example.com/)"
output_dir = "output"

[template]
slide = ""
index = ""
suffix = ""

[build]
theme_dir = ".marp/themes"
marp_binary = "marp"
```

### Add a New Slide

To add a new slide named `my-first-slide`:

```bash
slide-flow add my-first-slide
```

You can also specify if the slide should be `secret` (generated with a UUID-based path) or a `draft` (not published by default):

```bash
slide-flow add my-secret-slide --secret true --draft false
slide-flow add my-draft-slide --draft true
```

This will create a new directory `src/my-first-slide/` with `slide.md` and `slide.toml`.

### Prepare for Commit (Pre-Commit)

Run this command to update the `README.md` and `index.html` with the latest slide list and clean up old build caches.

```bash
slide-flow pre-commit
```

### Index Slides

This command puts slide numbers into your markdown files and generates a table of contents.
You can specify a single slide or process all of them.

```bash
# Index a specific slide
slide-flow index --dir src/my-first-slide

# Index all slides
slide-flow index
```

### Build Slides

Compile your slides into `HTML` and `PDF` formats. Specify the directories containing the `slide.md` files.

```bash
# Build a specific slide
slide-flow build src/my-first-slide

# Build multiple slides
slide-flow build src/my-first-slide src/another-slide

# Build with a custom concurrency limit (default is 4)
slide-flow build src/my-first-slide --concurrent 8
```

The built output will be placed in the directory specified by `output_dir` in your `config.toml` (default: `output/`).

## Project Structure

A typical `slide-flow` project looks like this:

```
.
├── .marp/
│   └── themes/             # Marp themes
│       └── my-theme.css
├── src/
│   ├── slide1/             # Individual slide directory
│   │   ├── images/         # Images specific to slide1
│   │   │   └── .gitkeep
│   │   ├── slide.md        # Marp markdown for slide1
│   │   └── slide.toml      # Configuration for slide1
│   └── slide2/
│       └── ...
├── output/                 # Default output directory for built slides
│   ├── index.html          # Main index page (generated)
│   ├── slide1/
│   │   └── index.html      # HTML output for slide1
│   └── slide1.pdf          # PDF output for slide1
└── config.toml             # Project-wide configuration
```

## Configuration

The `config.toml` file at the root of your project controls `slide-flow`'s behavior.

```toml
# Example config.toml
name = "My Awesome Slides"      # Project title
author = "Kenta Komoto"         # Your name
base_url = "[https://slides.example.com/](https://slides.example.com/)" # Base URL for generated links
output_dir = "public"           # Directory for compiled output

[template]
slide = "# New Slide\n\n\n\n" # Default content for new slide.md
index = "<h1>My Slides</h1>"    # Default content for index.html (when generated)
suffix = ""   # Content appended to slides (e.g., for global footers)

[build]
theme_dir = ".marp/themes"      # Path to your Marp themes
marp_binary = "marp"            # Command to invoke Marp CLI
```

Each slide can also have its own `slide.toml` file, e.g., `src/slide1/slide.toml`:

```toml
# Example slide.toml
name = "Introduction to Graph Algorithms" # Display name of the slide
version = 1                              # Slide version
secret = "some-uuid-here"                # Optional UUID for secret access
custom_path = ["graph-algos"]            # Optional custom URL path segments
draft = false                            # Set to true to exclude from public builds
description = "An overview of common graph algorithms." # Description for index pages
title_prefix = "# "                      # Marp title prefix (e.g., for indexing)
```

## Contributing

Contributions are welcome\! If you find a bug or have a feature request, please open an issue or submit a pull request on the [GitHub repository](https://www.google.com/search?q=https://github.com/kentakom1213/slide-flow).

## License

`slide-flow` is licensed under the MIT License. See the `LICENSE` file for details.
