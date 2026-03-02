use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

use crate::{
    core::config::Config,
    routes::app::{AppRouter, AppState},
};

pub type AppResult<T> = anyhow::Result<T>;

pub struct App;

impl App {
    pub async fn run() -> AppResult<()> {
        let config = Config::new()?;

        let db_pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await?;

        if config.auto_apply_migrations {
            sqlx::migrate!().run(&db_pool).await?
        }

        let state = AppState { config, db_pool };
        let app = AppRouter::new(state);

        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        println!("Server running on port 8000");
        axum::serve(listener, app).await.unwrap();

        Ok(())
    }
}
