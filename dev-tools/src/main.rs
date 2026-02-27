mod cli;
mod dev;
mod docs;

use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Dev { services } => dev::run(services).await?,
        Command::Docs => docs::run().await?,
        Command::DocsIndex => docs::generate_index()?,
    }

    Ok(())
}
