//! Application entry point and server bootstrap.
//!
//! This module provides the [`App`] struct with a single async method,
//! [`App::run`], that orchestrates the full server startup sequence: logging
//! initialization, configuration loading, database connection, optional
//! migrations, email client setup, and HTTP listener binding.

use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use crate::{
    core::{config::Config, logger::Logger},
    email::client::EmailClient,
    routes::app::{AppRouter, AppState},
};

/// Convenience alias for fallible operations during application startup.
pub type AppResult<T> = anyhow::Result<T>;

/// Entry point for the GigLog API server.
///
/// `App` is a unit struct whose only purpose is to namespace the [`run`](App::run)
/// method.
pub struct App;

impl App {
    /// Starts the API server.
    ///
    /// # Startup sequence
    ///
    /// 1. Initialize logging from the `LOG_LEVEL` environment variable.
    /// 2. Load [`Config`] from the environment.
    /// 3. Re-configure logging with the resolved config values.
    /// 4. Connect to PostgreSQL (max 5 connections).
    /// 5. Optionally apply pending SQLx migrations when
    ///    [`Config::auto_apply_migrations`] is `true`.
    /// 6. Create the [`EmailClient`].
    /// 7. Build [`AppState`] and [`AppRouter`].
    /// 8. Bind a TCP listener on `0.0.0.0:8000` and serve requests.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid, the database is
    /// unreachable, migrations fail, or the TCP listener cannot bind.
    pub async fn run() -> AppResult<()> {
        Logger::setup_logging_from_env();

        let config = Config::new()?;
        Logger::setup_logging(&config.log_level, config.log_verbose);

        Logger::log_message("Connecting to database");

        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await?;

        Logger::log_success("Database connection established");

        if config.auto_apply_migrations {
            Logger::log_message("Checking for pending database migrations");
            sqlx::migrate!().run(&db_pool).await?;
            Logger::log_success("Database migrations are up to date");
        }

        let email_client = EmailClient::new(&config);

        let state = AppState {
            config,
            db_pool,
            email_client,
        };
        let app = AppRouter::new(state);

        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        Logger::log_success("Server running on port 8000");
        axum::serve(listener, app).await?;

        Ok(())
    }
}
