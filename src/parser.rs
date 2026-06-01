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
        /// skip image optimization before building
        #[clap(long)]
        no_optimize_images: bool,
    },
    /// Image operations
    #[clap(arg_required_else_help = true)]
    Images {
        #[clap(subcommand)]
        command: ImagesCommands,
    },
    /// Slide operations
    #[clap(arg_required_else_help = true)]
    Slide {
        #[clap(subcommand)]
        command: SlidesCommands,
    },
    /// Migration operations
    #[clap(arg_required_else_help = true)]
    Migrate {
        #[clap(subcommand)]
        command: MigrateCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ImagesCommands {
    /// Optimize images referenced by a slide
    Optimize {
        /// path to slide directory
        #[clap(required = true)]
        dir: PathBuf,
        /// show what would be optimized without writing files
        #[clap(long)]
        dry_run: bool,
        /// ignore cached optimized files
        #[clap(long)]
        force: bool,
    },
    /// Optimize images referenced by all slides
    OptimizeAll {
        /// show what would be optimized without writing files
        #[clap(long)]
        dry_run: bool,
        /// ignore cached optimized files
        #[clap(long)]
        force: bool,
    },
    /// Remove optimized image cache
    Clean,
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

#[derive(Debug, Subcommand)]
pub enum MigrateCommands {
    /// Show planned migration changes
    Plan {
        /// slide directory (e.g. src/intro)
        dir: Option<PathBuf>,
    },
    /// Show migration status
    Status,
    /// Apply migration changes
    Apply {
        /// slide directory (e.g. src/intro)
        #[clap(required = true)]
        dir: PathBuf,
        /// update slide.toml only
        #[clap(long)]
        metadata_only: bool,
        /// generate redirects only
        #[clap(long)]
        redirects_only: bool,
        /// build canonical artifacts and redirects
        #[clap(long)]
        artifacts: bool,
        /// remove legacy alias artifacts
        #[clap(long)]
        remove_legacy_artifacts: bool,
        /// max concurrent build
        #[clap(long, default_value = "4")]
        concurrent: usize,
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

    #[test]
    fn parses_migrate_apply_command() {
        let cmd = Cmd::try_parse_from([
            "slide-flow",
            "migrate",
            "apply",
            "src/intro",
            "--metadata-only",
        ])
        .unwrap();

        match cmd.subcommand {
            SubCommands::Migrate {
                command:
                    MigrateCommands::Apply {
                        dir,
                        metadata_only,
                        redirects_only,
                        artifacts,
                        remove_legacy_artifacts,
                        concurrent,
                    },
            } => {
                assert_eq!(dir, PathBuf::from("src/intro"));
                assert!(metadata_only);
                assert!(!redirects_only);
                assert!(!artifacts);
                assert!(!remove_legacy_artifacts);
                assert_eq!(concurrent, 4);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
