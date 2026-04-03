//! Internal implementation for shared API integration-test helpers.
//!
//! Exposes deterministic setup utilities for configuration, database pools,
//! application state, router construction, and test data cleanup.

use std::env;

use axum::Router;
use gig_log_api::{
    core::config::Config,
    email::client::EmailClient,
    routes::app::{AppRouter, AppState},
};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use tokio::sync::OnceCell;

/// Environment variable containing the PostgreSQL connection URL for tests.
const TEST_DATABASE_URL_ENV: &str = "TEST_DATABASE_URL";
/// Optional environment variable overriding the email API base URL for tests.
const TEST_RESEND_BASE_URL_ENV: &str = "TEST_RESEND_BASE_URL";
/// Default wiremock URL used when `TEST_RESEND_BASE_URL` is not set.
const DEFAULT_TEST_RESEND_BASE_URL: &str = "http://127.0.0.1:4010";
/// One-time gate that ensures SQLx migrations run once per test process.
static TEST_MIGRATIONS_READY: OnceCell<()> = OnceCell::const_new();

/// Returns deterministic configuration values for integration tests.
///
/// # Returns
///
/// A [`Config`] with stable test values and `TEST_DATABASE_URL`.
pub fn test_config() -> Config {
    Config {
        app_env: "test".to_string(),
        web_origin: "http://localhost:3000".to_string(),
        database_url: test_database_url(),
        auto_apply_migrations: false,
        jwt_secret: "integration-test-jwt-secret".to_string(),
        jwt_access_token_expiry_seconds: 60,
        jwt_refresh_token_expiry_seconds: 120,
        resend_api_key: "re_test_api_key".to_string(),
        resend_from_email: "noreply@test.gig-log.local".to_string(),
        auth_code_expiry_seconds: 60,
        log_level: "debug".to_string(),
        log_verbose: false,
        log_http_max_body: 16384,
    }
}

/// Creates a PostgreSQL pool for integration tests and runs migrations.
///
/// # Returns
///
/// A connected [`Pool<Postgres>`] targeting `TEST_DATABASE_URL`.
pub async fn test_db_pool() -> Pool<Postgres> {
    let database_url = test_database_url();

    run_test_migrations(&database_url).await;

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap_or_else(|error| {
            panic!(
                "Failed to connect to test database '{}': {error}",
                database_url
            )
        })
}

/// Builds [`AppState`] for integration tests.
///
/// Uses [`test_config`], [`test_db_pool`], and an [`EmailClient`] pointed at
/// `TEST_RESEND_BASE_URL` (or a default wiremock URL).
///
/// # Returns
///
/// A fully initialized [`AppState`] for route testing.
pub async fn test_app_state() -> AppState {
    let config = test_config();
    let db_pool = test_db_pool().await;
    let email_client = EmailClient::new_with_base_url(&config, test_resend_base_url());

    AppState {
        config,
        db_pool,
        email_client,
    }
}

/// Builds the full application router for integration tests.
///
/// # Returns
///
/// A fully configured [`Router`] produced by [`AppRouter`].
pub async fn test_router() -> Router {
    AppRouter::new(test_app_state().await)
}

/// Truncates auth-related tables between integration tests.
///
/// Clears `users`, `auth_codes`, and `refresh_tokens` with `CASCADE` so tests
/// can run in isolation without leaking data between cases.
///
/// # Arguments
///
/// * `pool` — The test database [`Pool<Postgres>`].
pub async fn cleanup_auth_tables(pool: &Pool<Postgres>) {
    sqlx::query("TRUNCATE TABLE users, auth_codes, refresh_tokens RESTART IDENTITY CASCADE")
        .execute(pool)
        .await
        .unwrap_or_else(|error| {
            panic!("Failed to truncate test tables (users/auth_codes/refresh_tokens): {error}")
        });
}

/// Reads and validates the test database URL from the environment.
///
/// # Returns
///
/// A non-empty PostgreSQL URL from `TEST_DATABASE_URL`.
fn test_database_url() -> String {
    env::var(TEST_DATABASE_URL_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            panic!(
                "{TEST_DATABASE_URL_ENV} must be set for integration tests (for example: postgres://postgres:postgres@localhost:5432/gig-log-test)"
            )
        })
}

/// Resolves the email API base URL used in integration tests.
///
/// # Returns
///
/// A URL from `TEST_RESEND_BASE_URL`, or a default local wiremock URL.
fn test_resend_base_url() -> String {
    env::var(TEST_RESEND_BASE_URL_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_TEST_RESEND_BASE_URL.to_string())
}

/// Runs SQLx migrations once for the configured test database.
///
/// # Arguments
///
/// * `database_url` — The PostgreSQL URL for the test database.
async fn run_test_migrations(database_url: &str) {
    let database_url = database_url.to_string();

    TEST_MIGRATIONS_READY
        .get_or_init(|| async move {
            let migration_pool = PgPoolOptions::new()
                .max_connections(1)
                .connect(&database_url)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to connect to test database '{}' for migrations: {error}",
                        database_url
                    )
                });

            sqlx::migrate!()
                .run(&migration_pool)
                .await
                .unwrap_or_else(|error| {
                    panic!(
                        "Failed to apply migrations to test database '{}': {error}",
                        database_url
                    )
                });

            migration_pool.close().await;
        })
        .await;
}
