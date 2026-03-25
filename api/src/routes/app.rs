use axum::{
    Router,
    http::{HeaderName, Method},
    middleware,
};
use log::error;
use sqlx::{Pool, Postgres};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::{
    core::{
        config::Config,
        logger::{HttpLoggingConfig, Logger},
    },
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
        let http_logging_config = HttpLoggingConfig::new(
            state.config.is_http_body_logging_enabled(),
            state.config.log_http_max_body,
            state.config.log_verbose,
        );

        let web_origin = state.config.web_origin.parse().unwrap_or_else(|error| {
            error!(
                "Failed to parse WEB_ORIGIN '{}': {}; using http://localhost:3000 instead",
                state.config.web_origin, error
            );

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
            .layer(middleware::from_fn_with_state(
                http_logging_config,
                Logger::log_request_and_response,
            ))
            .layer(cors)
            .with_state(state)
    }
}
