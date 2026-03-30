//! Binary entry point for GigLog development tooling.
//!
//! This crate parses command-line arguments and dispatches subcommands that
//! run local workflows for the GigLog workspace.
//!
//! # Modules
//!
//! - [`api_tester`]: Interactive terminal client for testing API routes.
//! - [`cli`]: Command-line argument parser types.
//! - [`db_viewer`]: Terminal database query and table inspection tool.
//! - [`dev`]: Development orchestrator for running and rebuilding services.
//! - [`docs`]: Rustdoc build, index generation, and local docs serving.
//! - [`setup`]: Environment bootstrap workflow for local development.
//! - [`utils`]: Shared helper modules used by multiple commands.

mod api_tester;
mod cli;
mod db_viewer;
mod dev;
mod docs;
mod setup;
mod utils;

use clap::Parser;

use cli::{Cli, Command};

/// Runs the `gig-log-dev-tools` command dispatcher.
///
/// Loads workspace environment variables, parses CLI arguments, and executes
/// the selected subcommand workflow.
///
/// # Returns
///
/// An empty [`anyhow::Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if environment loading fails or a selected
/// subcommand returns an error.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    utils::env::load_workspace_env()?;

    let cli = Cli::parse();

    match cli.command {
        Command::Dev => dev::run().await?,
        Command::Docs => docs::run().await?,
        Command::DocsIndex => docs::generate_index()?,
        Command::Setup {
            non_interactive,
            skip_db,
            skip_migrate,
            skip_build,
            dry_run,
            release,
        } => {
            setup::run(setup::SetupOptions {
                non_interactive,
                skip_db,
                skip_migrate,
                skip_build,
                dry_run,
                release,
            })
            .await?
        }
        Command::ApiTester => api_tester::run().await?,
        Command::DbViewer => db_viewer::run().await?,
    }

    Ok(())
}
