//! parser of command line arguments

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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
        #[command(flatten)]
        targets: RequiredTargetArgs,
        /// max concurrent build
        #[clap(long, default_value = "4")]
        concurrent: usize,
        /// skip image optimization before building
        #[clap(long)]
        no_optimize_images: bool,
    },
    /// Prepare slides for publishing
    Prepare {
        #[command(flatten)]
        targets: OptionalTargetArgs,
        /// skip project README and index refresh
        #[clap(long)]
        no_refresh: bool,
        /// skip stale output cleanup
        #[clap(long)]
        no_clean: bool,
        /// skip table of contents updates
        #[clap(long)]
        no_toc: bool,
        /// skip bibliography updates
        #[clap(long)]
        no_bib: bool,
        /// skip builds
        #[clap(long)]
        no_build: bool,
        /// skip image optimization before building
        #[clap(long)]
        no_optimize_images: bool,
        /// max concurrent build
        #[clap(long, default_value = "4")]
        concurrent: usize,
        /// show planned steps without writing files
        #[clap(long)]
        dry_run: bool,
    },
    /// Put table of contents into slides
    Toc {
        #[command(flatten)]
        targets: RequiredTargetArgs,
        /// run quietly
        #[clap(short, long)]
        quiet: bool,
    },
    /// Update slide bibliography
    Bib {
        #[command(flatten)]
        targets: RequiredTargetArgs,
    },
    /// Clean generated files
    #[clap(arg_required_else_help = true)]
    Clean {
        #[clap(subcommand)]
        command: CleanCommands,
    },
    /// Project operations
    #[clap(arg_required_else_help = true)]
    Project {
        #[clap(subcommand)]
        command: ProjectCommands,
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

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
pub struct RequiredTargetArgs {
    /// path to slide directories
    #[clap(value_name = "DIR")]
    pub directories: Vec<PathBuf>,
    /// target all managed slides
    #[clap(long)]
    pub all: bool,
    /// target slides changed in Git
    #[clap(long)]
    pub changed: bool,
}

#[derive(Debug, Clone, Args, Default)]
#[group(multiple = false)]
pub struct OptionalTargetArgs {
    /// path to slide directories
    #[clap(value_name = "DIR")]
    pub directories: Vec<PathBuf>,
    /// target all managed slides
    #[clap(long)]
    pub all: bool,
    /// target slides changed in Git
    #[clap(long)]
    pub changed: bool,
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommands {
    /// List managed slides
    List,
    /// Show project configuration and basic information
    Show,
    /// Update README.md and output index.html
    Refresh,
}

#[derive(Debug, Subcommand)]
pub enum CleanCommands {
    /// Remove stale generated outputs
    Outputs {
        /// show what would be removed without deleting files
        #[clap(long)]
        dry_run: bool,
    },
    /// Remove stale generated outputs and image optimization cache
    All {
        /// show what would be removed without deleting files
        #[clap(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ImagesCommands {
    /// Optimize images referenced by slides
    Optimize {
        #[command(flatten)]
        targets: RequiredTargetArgs,
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
        #[clap(long, conflicts_with = "public")]
        secret: bool,
        /// Make public page
        #[clap(long, conflicts_with = "secret")]
        public: bool,
        /// Make draft page
        #[clap(long)]
        draft: bool,
        /// Slide type. By default `marp`
        #[clap(long = "type")]
        type_: Option<SlideType>,
    },
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
                        public,
                        draft,
                        type_: None,
                    },
            } => {
                assert_eq!(name, "intro");
                assert!(!secret);
                assert!(!public);
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
    fn parses_project_list_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "project", "list"]).unwrap();

        match cmd.subcommand {
            SubCommands::Project {
                command: ProjectCommands::List,
            } => {}
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_toc_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "toc", "src/intro", "--quiet"]).unwrap();

        match cmd.subcommand {
            SubCommands::Toc { targets, quiet } => {
                assert_eq!(targets.directories, vec![PathBuf::from("src/intro")]);
                assert!(!targets.all);
                assert!(!targets.changed);
                assert!(quiet);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_bib_all_command() {
        let cmd = Cmd::try_parse_from(["slide-flow", "bib", "--all"]).unwrap();

        match cmd.subcommand {
            SubCommands::Bib { targets } => {
                assert!(targets.directories.is_empty());
                assert!(targets.all);
                assert!(!targets.changed);
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn rejects_build_without_target() {
        let err = Cmd::try_parse_from(["slide-flow", "build"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn rejects_multiple_target_modes() {
        let err = Cmd::try_parse_from(["slide-flow", "build", "src/intro", "--all"]).unwrap_err();

        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn parses_prepare_without_target() {
        let cmd = Cmd::try_parse_from(["slide-flow", "prepare", "--dry-run"]).unwrap();

        match cmd.subcommand {
            SubCommands::Prepare {
                targets, dry_run, ..
            } => {
                assert!(targets.directories.is_empty());
                assert!(!targets.all);
                assert!(!targets.changed);
                assert!(dry_run);
            }
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
