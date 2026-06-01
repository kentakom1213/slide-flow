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
    /// Build slide
    Build {
        /// path to slide directory
        #[clap(required = true)]
        directories: Vec<PathBuf>,
        /// max concurrent build
        #[clap(long, default_value = "4")]
        concurrent: usize,
    },
    /// Slide operations
    #[clap(arg_required_else_help = true)]
    Slide {
        #[clap(subcommand)]
        command: SlidesCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum SlidesCommands {
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
    /// List managed slides
    List,
    /// Show slide details by list number or path
    Show {
        /// slide number from `slides list` or slide path like `src/intro`
        selector: String,
    },
    /// bump slide version and archive current contents
    Archive {
        /// slide directory (e.g. src/intro)
        #[clap(required = true)]
        dir: PathBuf,
    },
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_slide_show_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "slide", "show", "1"]).unwrap();

        match cmd.subcommand {
            SubCommands::Slide {
                command: SlidesCommands::Show { selector },
            } => assert_eq!(selector, "1"),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_slide_add_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "slide", "add", "intro"]).unwrap();

        match cmd.subcommand {
            SubCommands::Slide {
                command:
                    SlidesCommands::Add {
                        name,
                        secret,
                        draft,
                        type_: None,
                    },
            } => {
                assert_eq!(name, "intro");
                assert!(secret);
                assert!(!draft);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_slide_archive_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "slide", "archive", "src/intro"]).unwrap();

        match cmd.subcommand {
            SubCommands::Slide {
                command: SlidesCommands::Archive { dir },
            } => assert_eq!(dir, PathBuf::from("src/intro")),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_slide_index_command() {
        let cmd = Cmd::try_parse_from([
            "slide-flow",
            "slide",
            "index",
            "--dir",
            "src/intro",
            "--quiet",
        ])
        .unwrap();

        match cmd.subcommand {
            SubCommands::Slide {
                command: SlidesCommands::Index { dir, quiet },
            } => {
                assert_eq!(dir, Some(PathBuf::from("src/intro")));
                assert!(quiet);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_slide_bib_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "slide", "bib", "src/intro"]).unwrap();

        match cmd.subcommand {
            SubCommands::Slide {
                command: SlidesCommands::Bib { dir },
            } => assert_eq!(dir, PathBuf::from("src/intro")),
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn slide_command_requires_subcommand_or_selector() {
        let err = Cmd::try_parse_from(["slide-flow", "slide"]).unwrap_err();

        assert_eq!(
            err.kind(),
            clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }
}
