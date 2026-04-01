//! Application router and shared state for the GigLog API.
//!
//! This module defines [`AppState`], the shared state available to all
//! request handlers, and [`AppRouter`], which assembles every route group,
//! configures CORS, and applies HTTP logging middleware.

use axum::{
    Router,
    http::{HeaderName, HeaderValue, Method},
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

/// Shared application state passed to every Axum handler.
///
/// Holds the runtime configuration, a PostgreSQL connection pool, and an
/// email client. Axum clones this state for each request via its [`Clone`]
/// implementation.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Runtime application configuration loaded from environment variables.
    pub config: Config,
    /// PostgreSQL connection pool for database operations.
    pub db_pool: Pool<Postgres>,
    /// Email client for sending transactional emails.
    pub email_client: EmailClient,
}

/// Top-level router builder for the GigLog API.
pub struct AppRouter;

impl AppRouter {
    /// Creates a fully configured [`Router`] with all route groups and middleware.
    ///
    /// Parses configured web origins for CORS, falling back to
    /// `http://localhost:3000` when none are valid. Nests [`HealthRouter`]
    /// at `/health` and [`AuthRouter`] at `/auth`, then applies HTTP request/response
    /// logging and CORS middleware layers.
    ///
    /// # Arguments
    ///
    /// * `state` — The [`AppState`] shared across all handlers.
    ///
    /// # Returns
    ///
    /// A configured [`Router`] ready to serve requests.
    pub fn new(state: AppState) -> Router {
        let http_logging_config = HttpLoggingConfig::new(
            state.config.is_http_body_logging_enabled(),
            state.config.log_http_max_body,
            state.config.log_verbose,
        );
        let allowed_origins = Self::build_allowed_origins(&state.config.web_origins);

        let cors = CorsLayer::new()
            .allow_origin(allowed_origins)
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

    /// Builds a validated CORS allow-list from configured origins.
    ///
    /// # Arguments
    ///
    /// * `web_origins` — Configured origins from [`Config`](crate::core::config::Config).
    ///
    /// # Returns
    ///
    /// An [`AllowOrigin`] containing all valid origins or a localhost fallback.
    fn build_allowed_origins(web_origins: &[String]) -> AllowOrigin {
        let mut parsed_origins = web_origins
            .iter()
            .filter_map(|origin| {
                let trimmed_origin = origin.trim();

                match HeaderValue::from_str(trimmed_origin) {
                    Ok(value) => Some(value),
                    Err(error) => {
                        error!(
                            "Failed to parse WEB_ORIGIN value '{}': {}; skipping",
                            origin, error
                        );

                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        if parsed_origins.is_empty() {
            error!("No valid WEB_ORIGIN values found; using http://localhost:3000 instead");
            parsed_origins.push(HeaderValue::from_static("http://localhost:3000"));
        }

        AllowOrigin::list(parsed_origins)
    }
}
