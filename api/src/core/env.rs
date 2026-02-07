//! Environment configuration loading utilities.
//!
//! This module reads required and optional environment variables, applies
//! sensible defaults, and produces the runtime configuration used by the API.

use std::env;

use anyhow::Error;
use dotenvy::dotenv;

use crate::core::app::AppResult;

/// Runtime configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Env {
    /// PostgreSQL connection string used by SQLx.
    pub database_url: String,
    /// Allowed CORS origin for browser requests.
    pub cors_allowed_origin: String,
    /// TCP port for the HTTP server.
    pub port: u16,
    /// Secret key used to sign and verify JWTs.
    pub jwt_secret: String,
    /// Access token lifetime in seconds.
    pub jwt_access_token_expiry_seconds: u64,
    /// Refresh token lifetime in seconds.
    pub jwt_refresh_token_expiry_seconds: u64,
    /// Resend API key for transactional emails.
    pub resend_api_key: String,
    /// Sender email used by the Resend integration.
    pub resend_from_email: String,
    /// Authentication code lifetime in seconds.
    pub auth_code_expiry_seconds: u64,
    /// Cookie domain used when setting auth cookies.
    pub cookie_domain: String,
    /// Whether auth cookies are marked as `Secure`.
    pub cookie_secure: bool,
}

impl Env {
    /// Loads application configuration from environment variables.
    ///
    /// Required variables:
    /// - `DATABASE_URL`
    /// - `JWT_SECRET`
    /// - `RESEND_API_KEY`
    /// - `RESEND_FROM_EMAIL`
    ///
    /// Optional variables fall back to defaults when unset or empty.
    ///
    /// # Errors
    ///
    /// Returns an error if a required variable is missing or if a numeric
    /// environment variable cannot be parsed.
    pub fn new() -> AppResult<Self> {
        dotenv().ok();

        let database_url = Self::get_required_var("DATABASE_URL")?;

        let cors_allowed_origin = match Self::get_optional_var("CORS_ALLOWED_ORIGIN") {
            Some(cors_allowed_origin) => cors_allowed_origin,
            None => "http://localhost:3000".to_string(),
        };

        let port = match Self::get_optional_var("PORT") {
            Some(port) => port.trim().parse::<u16>()?,
            None => 8000,
        };

        // JWT Configuration
        let jwt_secret = Self::get_required_var("JWT_SECRET")?;

        let jwt_access_token_expiry_seconds =
            match Self::get_optional_var("JWT_ACCESS_TOKEN_EXPIRY_SECONDS") {
                Some(val) => val.trim().parse::<u64>()?,
                None => 900, // 15 minutes
            };

        let jwt_refresh_token_expiry_seconds =
            match Self::get_optional_var("JWT_REFRESH_TOKEN_EXPIRY_SECONDS") {
                Some(val) => val.trim().parse::<u64>()?,
                None => 604800, // 7 days
            };

        // Resend Email Service
        let resend_api_key = Self::get_required_var("RESEND_API_KEY")?;
        let resend_from_email = Self::get_required_var("RESEND_FROM_EMAIL")?;

        // Auth Codes
        let auth_code_expiry_seconds = match Self::get_optional_var("AUTH_CODE_EXPIRY_SECONDS") {
            Some(val) => val.trim().parse::<u64>()?,
            None => 600, // 10 minutes
        };

        // Cookie Configuration
        let cookie_domain = match Self::get_optional_var("COOKIE_DOMAIN") {
            Some(val) => val,
            None => "localhost".to_string(),
        };

        let cookie_secure = match Self::get_optional_var("COOKIE_SECURE") {
            Some(val) => val.trim().to_lowercase() == "true",
            None => false,
        };

        Ok(Self {
            database_url,
            cors_allowed_origin,
            port,
            jwt_secret,
            jwt_access_token_expiry_seconds,
            jwt_refresh_token_expiry_seconds,
            resend_api_key,
            resend_from_email,
            auth_code_expiry_seconds,
            cookie_domain,
            cookie_secure,
        })
    }

    /// Reads a required environment variable.
    ///
    /// # Arguments
    ///
    /// - `var` - Environment variable name.
    ///
    /// # Errors
    ///
    /// Returns an error when the variable is missing or empty.
    fn get_required_var(var: &str) -> AppResult<String> {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => Ok(value),
            _ => {
                let error_message = format!("`{}` environment variable not set.", var);

                Err(Error::msg(error_message))
            }
        }
    }

    /// Reads an optional environment variable.
    ///
    /// # Arguments
    ///
    /// - `var` - Environment variable name.
    ///
    /// Returns `None` when the variable is missing or empty.
    fn get_optional_var(var: &str) -> Option<String> {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        }
    }
}
