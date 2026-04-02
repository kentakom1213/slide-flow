//! parser of command line arguments

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::SlideType;

/// command line arguments
#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
pub struct Cmd {
    #[clap(subcommand)]
    pub subcommand: SubCommands,
}

#[derive(Debug, Subcommand)]
pub enum SubCommands {
    /// Initialize project
    Init,
    /// Create new slide
    #[clap(arg_required_else_help = true)]
    Add {
        /// slide name
        #[clap(required = true)]
        name: String,
        /// Make secret page
        #[clap(long, default_value_t = true)]
        secret: bool,
        /// Make draft page
        #[clap(long, default_value_t = false)]
        draft: bool,
        /// Slide type. By default `marp`
        #[clap(long = "type")]
        type_: Option<SlideType>,
    },
    /// Prepare slides for build
    PreCommit,
    /// Put index to slide
    Index {
        /// specify slide directory
        #[clap(short, long)]
        dir: Option<PathBuf>,
        /// run quietly
        #[clap(short, long)]
        quiet: bool,
    },
    /// Modify slide bibliography
    Bib {
        /// slide directory
        dir: PathBuf,
    },
    /// Build slide
    Build {
        /// path to slide directory
        #[clap(required = true)]
        directories: Vec<PathBuf>,
        /// max concurrent build
        #[clap(long, default_value = "4")]
        concurrent: usize,
    },
    /// Slide version operations
    Version {
        #[clap(subcommand)]
        command: VersionCommands,
    },
    /// Slide operations
    Slides {
        #[clap(subcommand)]
        command: SlidesCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum VersionCommands {
    /// bump slide version and archive current contents
    Bump {
        /// slide directory (e.g. src/intro)
        #[clap(required = true)]
        dir: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum SlidesCommands {
    /// List managed slides
    List,
    /// Show slide details by list number or path
    Detail {
        /// slide number from `slides list` or slide path like `src/intro`
        selector: String,
    },
}
