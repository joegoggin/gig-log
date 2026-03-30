//! Command-line argument definitions for `gig-log-dev-tools`.
//!
//! This module defines the top-level CLI parser and the available subcommands
//! used to run development workflows.

use clap::{Parser, Subcommand};

/// Parses top-level command-line input for `gig-log-dev-tools`.
#[derive(Parser)]
#[command(name = "gig-log-dev-tools", about = "Development tools for gig-log")]
pub struct Cli {
    /// Selected subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Enumerates all supported `gig-log-dev-tools` subcommands.
#[derive(Subcommand)]
pub enum Command {
    /// Starts the development orchestrator in TUI mode.
    Dev,
    /// Builds and serves workspace documentation.
    Docs,
    /// Generates the workspace documentation `index.html` file.
    DocsIndex,
    /// Initializes the local development environment.
    Setup {
        /// Disables interactive prompts and fails when required values are missing.
        #[arg(long)]
        non_interactive: bool,
        /// Skips starting database containers.
        #[arg(long)]
        skip_db: bool,
        /// Skips running SQLx migrations.
        #[arg(long)]
        skip_migrate: bool,
        /// Skips building workspace crates.
        #[arg(long)]
        skip_build: bool,
        /// Prints planned actions without executing commands.
        #[arg(long)]
        dry_run: bool,
        /// Builds the workspace in release mode.
        #[arg(long)]
        release: bool,
    },
    /// Launches the API tester TUI.
    ApiTester,
    /// Launches the database viewer TUI.
    DbViewer,
}
