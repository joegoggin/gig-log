//! Actix HTTP server setup and execution.
//!
//! This module configures the database pool, CORS middleware, shared app data,
//! and route registration for the API server.

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware::from_fn, web};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

use crate::core::{
    app::AppResult,
    app_state::AppState,
    config::configure_routes,
    env::Env,
    logger::{HttpLoggingConfig, Logger},
};

/// HTTP server with initialized shared dependencies.
pub struct Server {
    pool: Pool<Postgres>,
    env: Env,
}

impl Server {
    /// Creates a new server instance and initializes the PostgreSQL pool.
    ///
    /// # Arguments
    ///
    /// - `env` - Runtime environment configuration used for DB and HTTP setup.
    ///
    /// # Errors
    ///
    /// Returns an error if the database pool cannot connect.
    pub async fn new(env: Env) -> AppResult<Server> {
        Logger::log_message("Connecting to database");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env.database_url)
            .await?;

        Logger::log_success("Database connection established");

        Ok(Server { pool, env })
    }

    /// Starts the Actix HTTP server and blocks until shutdown.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the server cannot bind or the runtime
    /// encounters a fatal server error.
    pub async fn run(self) -> std::io::Result<()> {
        Logger::log_success(&format!("Server running on port {}", self.env.port));

        let env = self.env.clone();
        let app_state = AppState::new(self.pool.clone(), env.clone());
        let http_logging_config = HttpLoggingConfig {
            body_enabled: env.log_http_body_enabled,
            max_body_bytes: env.log_http_max_body_bytes,
        };

        HttpServer::new(move || {
            let cors = Cors::default()
                .allowed_origin(&env.cors_allowed_origin)
                .allowed_methods(vec!["GET", "PUT", "POST", "DELETE", "OPTIONS"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
                .supports_credentials();

            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .app_data(web::Data::new(http_logging_config.clone()))
                .wrap(cors)
                .wrap(from_fn(Logger::log_request_and_response))
                .configure(configure_routes)
        })
        .bind(("127.0.0.1", self.env.port))?
        .run()
        .await
    }
}
