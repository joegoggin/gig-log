use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gig-log-dev-tools", about = "Development tools for gig-log")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start the development orchestrator in TUI mode
    Dev,
    /// Build and serve workspace documentation
    Docs,
    /// Generate workspace documentation index.html
    DocsIndex,
    /// Initialize local development environment
    Setup {
        /// Do not prompt for input; fail if required values are missing
        #[arg(long)]
        non_interactive: bool,
        /// Skip starting database containers
        #[arg(long)]
        skip_db: bool,
        /// Skip running SQLx migrations
        #[arg(long)]
        skip_migrate: bool,
        /// Skip building workspace crates
        #[arg(long)]
        skip_build: bool,
        /// Print actions without executing commands
        #[arg(long)]
        dry_run: bool,
        /// Build in release mode
        #[arg(long)]
        release: bool,
    },
    // Add to the Command enum in dev-tools/src/cli.rs
    /// Launch the API tester TUI
    ApiTester,
}
