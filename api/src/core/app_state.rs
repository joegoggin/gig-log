//! Shared application state for HTTP handlers and middleware.
//!
//! This module groups runtime dependencies needed by request handlers so they
//! can receive a single injected state value in both production and tests.

use std::sync::Arc;

use sqlx::{Pool, Postgres};

use crate::core::env::Env;
use crate::services::email::{EmailSender, EmailService};

/// Shared email sender trait object used by handlers.
pub type DynEmailSender = Arc<dyn EmailSender + Send + Sync>;

/// Runtime application dependencies shared across requests.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool.
    pub pool: Pool<Postgres>,
    /// Runtime environment configuration.
    pub env: Env,
    /// Email sender used by authentication flows.
    pub email_sender: DynEmailSender,
}

impl AppState {
    /// Creates app state with the default Resend-backed email sender.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool.
    /// - `env` - Runtime environment configuration.
    pub fn new(pool: Pool<Postgres>, env: Env) -> Self {
        let email_sender = Arc::new(EmailService::new(
            &env.resend_api_key,
            &env.resend_from_email,
        ));

        Self {
            pool,
            env,
            email_sender,
        }
    }

    /// Creates app state with an injected email sender implementation.
    ///
    /// This is primarily used by tests to inject a mock sender.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool.
    /// - `env` - Runtime environment configuration.
    /// - `email_sender` - Email sender implementation.
    pub fn with_email_sender(pool: Pool<Postgres>, env: Env, email_sender: DynEmailSender) -> Self {
        Self {
            pool,
            env,
            email_sender,
        }
    }
}
