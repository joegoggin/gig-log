//! Application bootstrap entry point.
//!
//! This module wires together environment configuration and server startup.

use crate::core::{env::Env, server::Server};

/// Shared result type used by application startup helpers.
pub type AppResult<T> = anyhow::Result<T>;

/// Top-level application runner.
///
/// This type exposes the startup flow for loading configuration and launching
/// the HTTP server.
pub struct App;

impl App {
    /// Runs the API application.
    ///
    /// Loads environment configuration, initializes server dependencies,
    /// and starts the HTTP server loop.
    ///
    /// # Errors
    ///
    /// Returns an error if environment loading fails, database connection
    /// setup fails, or the server fails to start.
    pub async fn run() -> AppResult<()> {
        let env = Env::new()?;
        let server = Server::new(env).await?;

        server.run().await?;

        Ok(())
    }
}
