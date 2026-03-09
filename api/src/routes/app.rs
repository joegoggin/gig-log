use axum::{
    Router,
    http::{HeaderName, Method},
};
use sqlx::{Pool, Postgres};
use tower_http::cors::{AllowOrigin, CorsLayer};

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
        let web_origin = state.config.web_origin.parse().unwrap_or_else(|_| {
            "http://localhost:3000"
                .parse()
                .expect("default localhost origin should always be valid")
        });

        let cors = CorsLayer::new()
            .allow_origin(AllowOrigin::exact(web_origin))
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([
                HeaderName::from_static("content-type"),
                HeaderName::from_static("authorization"),
            ])
            .allow_credentials(true);

        Router::new()
            .nest("/health", HealthRouter::new())
            .nest("/auth", AuthRouter::new())
            .layer(cors)
            .with_state(state)
    }
}
