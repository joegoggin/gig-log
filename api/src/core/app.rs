use crate::core::{env::Env, server::Server};

pub type AppResult<T> = anyhow::Result<T>;

pub struct App;

impl App {
    pub async fn run() -> AppResult<()> {
        let env = Env::new()?;
        let server = Server::new(env).await?;

        server.run().await?;

        Ok(())
    }
}
