//! parser of command line arguments

use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
    /// initialize project
    Init,
    /// create new slide
    #[clap(arg_required_else_help = true)]
    Add {
        /// slide name
        #[clap(required = true)]
        name: String,
        /// make secret page
        #[clap(long, default_value_t = true)]
        secret: bool,
        /// make draft page
        #[clap(long, default_value_t = false)]
        draft: bool,
    },
    /// prepare slides for build
    PreCommit,
    /// put index to slide
    Index {
        /// specify slide directory
        #[clap(short, long)]
        dir: Option<PathBuf>,
        /// run quietly
        #[clap(short, long)]
        quiet: bool,
    },
    /// modify slide bibliography
    Bib {
        /// slide directory
        dir: PathBuf,
    },
    /// build slide
    Build {
        /// path to slide directory
        #[clap(required = true)]
        directories: Vec<PathBuf>,
        /// max concurrent build
        #[clap(long, default_value = "4")]
        concurrent: usize,
    },
}
