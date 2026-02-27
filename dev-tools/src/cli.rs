use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gig-log-dev-tools", about = "Development tools for gig-log")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start all services in TUI mode
    Dev {
        /// Comma-separated list of services to start (api,web,docs)
        #[arg(long, value_delimiter = ',')]
        services: Option<Vec<String>>,
    },
    /// Build and serve workspace documentation
    Docs,
    /// Generate workspace documentation index.html
    DocsIndex,
}
