//! Shared helpers for API integration tests.
//!
//! This module provides database setup utilities and a mock email sender so
//! route tests can validate auth flows without calling external services.

use std::env;
use std::sync::{Arc, Mutex};

use api::core::app_state::AppState;
use api::core::env::Env;
use api::core::error::ApiError;
use api::services::email::EmailSender;
use async_trait::async_trait;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use uuid::Uuid;

fn load_dotenv() {
    dotenvy::dotenv().ok();
}

/// Type of email sent by the mock sender.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockEmailKind {
    /// Account confirmation email.
    Confirmation,
    /// Password reset email.
    PasswordReset,
}

/// Captured email invocation for assertions in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockEmailCall {
    /// Email category that was sent.
    pub kind: MockEmailKind,
    /// Recipient email address.
    pub to_email: String,
    /// Recipient first name included in the template.
    pub first_name: String,
    /// One-time code included in the email.
    pub code: String,
}

/// In-memory email sender used by integration tests.
#[derive(Debug, Default)]
pub struct MockEmailSender {
    calls: Mutex<Vec<MockEmailCall>>,
}

impl MockEmailSender {
    /// Creates a new mock sender.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a snapshot of all captured email calls.
    pub fn calls(&self) -> Vec<MockEmailCall> {
        self.calls
            .lock()
            .expect("mock email mutex poisoned")
            .clone()
    }
}

#[async_trait]
impl EmailSender for MockEmailSender {
    async fn send_confirmation_email(
        &self,
        to_email: &str,
        first_name: &str,
        code: &str,
    ) -> Result<(), ApiError> {
        self.calls
            .lock()
            .expect("mock email mutex poisoned")
            .push(MockEmailCall {
                kind: MockEmailKind::Confirmation,
                to_email: to_email.to_string(),
                first_name: first_name.to_string(),
                code: code.to_string(),
            });

        Ok(())
    }

    async fn send_password_reset_email(
        &self,
        to_email: &str,
        first_name: &str,
        code: &str,
    ) -> Result<(), ApiError> {
        self.calls
            .lock()
            .expect("mock email mutex poisoned")
            .push(MockEmailCall {
                kind: MockEmailKind::PasswordReset,
                to_email: to_email.to_string(),
                first_name: first_name.to_string(),
                code: code.to_string(),
            });

        Ok(())
    }
}

/// Returns a shared test database pool and runs migrations once.
///
/// The test URL is resolved from `TEST_DATABASE_URL` and falls back to
/// `DATABASE_URL` when not set.
pub async fn test_pool() -> Pool<Postgres> {
    load_dotenv();

    let database_url = env::var("TEST_DATABASE_URL")
        .or_else(|_| env::var("DATABASE_URL"))
        .expect("set TEST_DATABASE_URL or DATABASE_URL for integration tests");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to run migrations for tests");

    pool
}

/// Builds a deterministic test environment config.
pub fn test_env() -> Env {
    load_dotenv();

    Env {
        app_env: "test".to_string(),
        docker_compose_auto_start_enabled: false,
        auto_apply_migrations_enabled: false,
        database_url: env::var("TEST_DATABASE_URL")
            .or_else(|_| env::var("DATABASE_URL"))
            .expect("set TEST_DATABASE_URL or DATABASE_URL for integration tests"),
        cors_allowed_origin: "http://localhost:3000".to_string(),
        port: 0,
        jwt_secret: "integration-test-jwt-secret".to_string(),
        jwt_access_token_expiry_seconds: 900,
        jwt_refresh_token_expiry_seconds: 604_800,
        resend_api_key: "test-resend-key".to_string(),
        resend_from_email: "test@giglog.dev".to_string(),
        auth_code_expiry_seconds: 600,
        cookie_domain: Some("localhost".to_string()),
        cookie_secure: false,
        log_level: "info".to_string(),
        log_http_body_enabled: true,
        log_http_max_body_bytes: 16_384,
    }
}

/// Creates app state using a mock email sender.
///
/// Returns the state plus the mock sender handle for assertions.
pub fn app_state_with_mock_email(pool: Pool<Postgres>) -> (AppState, Arc<MockEmailSender>) {
    let env = test_env();
    let email_sender = Arc::new(MockEmailSender::new());
    let app_state = AppState::with_email_sender(pool, env, email_sender.clone());

    (app_state, email_sender)
}

/// Builds a unique email address for isolated test data.
pub fn unique_email(prefix: &str) -> String {
    format!("{}-{}@example.com", prefix, Uuid::new_v4())
}
