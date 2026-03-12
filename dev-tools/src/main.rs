mod api_tester;
mod cli;
mod dev;
mod docs;
mod setup;
mod utils;

use clap::Parser;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
    }

    Ok(())
}
