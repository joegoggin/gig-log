use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use crate::{
    core::{config::Config, logger::Logger},
    email::client::EmailClient,
    routes::app::{AppRouter, AppState},
};

pub type AppResult<T> = anyhow::Result<T>;

pub struct App;

impl App {
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
