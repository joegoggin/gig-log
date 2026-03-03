use axum::Router;
use sqlx::{Pool, Postgres};

use crate::{
    core::config::Config,
    email::client::EmailClient,
    routes::{auth::AuthRouter, health::HealthRouter},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: Config,
    pub db_pool: Pool<Postgres>,
    pub email_client: EmailClient,
}

pub struct AppRouter;

impl AppRouter {
    pub fn new(state: AppState) -> Router {
        Router::new()
            .nest("/health", HealthRouter::new())
            .nest("/auth", AuthRouter::new())
            .with_state(state)
    }
}
