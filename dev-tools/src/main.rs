mod cli;
mod dev;
mod docs;
mod setup;

use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Dev { services } => dev::run(services).await?,
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
    }

    Ok(())
}
