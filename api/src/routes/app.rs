use axum::{Router, routing::get};
use sqlx::{Pool, Postgres};

use crate::{
    core::config::Config,
    routes::health::HealthRouter,
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: Pool<Postgres>,
}

pub struct AppRouter;

impl AppRouter {
    pub fn new(state: AppState) -> Router {
        Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .nest("/health", HealthRouter::new())
            .with_state(state)
    }
}
