//! Application configuration loaded from environment variables.
//!
//! This module provides the [`Config`] struct, which reads all required and
//! optional settings from the process environment (with `.env` file support via
//! `dotenvy`). Required variables cause a startup error when missing; optional
//! variables fall back to sensible defaults documented on each field.

use std::env;

use anyhow::Error;
use dotenvy::dotenv;
use log::error;

use crate::core::app::AppResult;

/// Runtime configuration for the API server.
///
/// Constructed once at startup via [`Config::new`]. All fields are populated
/// from environment variables; see individual field docs for the variable name,
/// whether it is required, and its default value.
#[derive(Debug, Clone)]
pub struct Config {
    /// Application environment name. **Required** — `APP_ENV`.
    pub app_env: String,
    /// Allowed web client origin for CORS. **Required** — `WEB_ORIGIN`.
    pub web_origin: String,
    /// PostgreSQL connection string. **Required** — `DATABASE_URL`.
    pub database_url: String,
    /// Run pending SQLx migrations on startup. `AUTO_APPLY_MIGRATIONS_ENABLED`, default `true`.
    pub auto_apply_migrations: bool,
    /// Secret key used to sign JWTs. **Required** — `JWT_SECRET`.
    pub jwt_secret: String,
    /// JWT access token lifetime in seconds. `JWT_ACCESS_TOKEN_EXPIRY_SECONDS`, default `900` (15 min).
    pub jwt_access_token_expiry_seconds: u64,
    /// JWT refresh token lifetime in seconds. `JWT_REFRESH_TOKEN_EXPIRY_SECONDS`, default `604800` (7 days).
    pub jwt_refresh_token_expiry_seconds: u64,
    /// Resend API key for transactional email. **Required** — `RESEND_API_KEY`.
    pub resend_api_key: String,
    /// Sender address for outgoing email. **Required** — `RESEND_FROM_EMAIL`.
    pub resend_from_email: String,
    /// Auth code expiry in seconds. `AUTH_CODE_EXPIRY_SECONDS`, default `600` (10 min).
    pub auth_code_expiry_seconds: u64,
    /// Log level filter string. `LOG_LEVEL`, default `"debug"`.
    pub log_level: String,
    /// Enable verbose (structured) log output. `LOG_VERBOSE`, default `true` in development.
    pub log_verbose: bool,
    /// Maximum HTTP body size in bytes to include in logs. `LOG_HTTP_MAX_BODY_BYTES`, default `16384`.
    pub log_http_max_body: usize,
}

impl Config {
    /// Loads configuration from the environment.
    ///
    /// Reads a `.env` file if present, then resolves every setting. Returns an
    /// error if any required variable is missing or empty.
    pub fn new() -> AppResult<Self> {
        dotenv().ok();

        let app_env = Self::get_var_from_env("APP_ENV")?;
        let web_origin = Self::get_var_from_env("WEB_ORIGIN")?;
        let database_url = Self::get_var_from_env("DATABASE_URL")?;
        let auto_apply_migrations = Self::get_optional_bool("AUTO_APPLY_MIGRATIONS_ENABLED", true);
        let jwt_secret = Self::get_var_from_env("JWT_SECRET")?;
        let jwt_access_token_expiry_seconds =
            Self::get_optional_number("JWT_ACCESS_TOKEN_EXPIRY_SECONDS", 900);
        let jwt_refresh_token_expiry_seconds =
            Self::get_optional_number("JWT_REFRESH_TOKEN_EXPIRY_SECONDS", 604800);
        let resend_api_key = Self::get_var_from_env("RESEND_API_KEY")?;
        let resend_from_email = Self::get_var_from_env("RESEND_FROM_EMAIL")?;
        let auth_code_expiry_seconds = Self::get_optional_number("AUTH_CODE_EXPIRY_SECONDS", 600);
        let log_level = Self::get_optional_string("LOG_LEVEL", "debug");
        let log_verbose =
            Self::get_optional_bool("LOG_VERBOSE", Self::is_development_env(&app_env));
        let log_http_max_body = Self::get_optional_usize("LOG_HTTP_MAX_BODY_BYTES", 16384);

        Ok(Self {
            app_env,
            web_origin,
            database_url,
            auto_apply_migrations,
            jwt_secret,
            jwt_access_token_expiry_seconds,
            jwt_refresh_token_expiry_seconds,
            resend_api_key,
            resend_from_email,
            auth_code_expiry_seconds,
            log_level,
            log_verbose,
            log_http_max_body,
        })
    }

    /// Returns `true` when [`app_env`](Self::app_env) indicates a development environment.
    pub fn is_development(&self) -> bool {
        Self::is_development_env(&self.app_env)
    }

    /// Returns `true` when HTTP request/response bodies should be included in logs.
    pub fn is_http_body_logging_enabled(&self) -> bool {
        self.is_development()
    }

    /// Returns `true` if the given environment string is `"development"` or `"dev"` (case-insensitive).
    pub fn is_development_env(app_env: &str) -> bool {
        let normalized = app_env.trim().to_lowercase();
        normalized == "development" || normalized == "dev"
    }

    /// Reads a required, non-empty environment variable or returns an error.
    fn get_var_from_env(var: &str) -> AppResult<String> {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => Ok(value),
            _ => {
                let error_message = format!("`{}` environment variable not set.", var);

                Err(Error::msg(error_message))
            }
        }
    }

    /// Reads an optional string variable, falling back to `default`.
    fn get_optional_string(var: &str, default: impl Into<String>) -> String {
        match Self::get_var_from_env(var) {
            Ok(value) => value,
            Err(_) => default.into(),
        }
    }

    /// Reads an optional boolean variable, falling back to `default`.
    fn get_optional_bool(var: &str, default: bool) -> bool {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => {
                match value.trim().to_ascii_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => true,
                    "false" | "0" | "no" | "off" => false,
                    _ => {
                        error!(
                            "Invalid boolean value for {}='{}'; using default {}",
                            var, value, default
                        );
                        default
                    }
                }
            }
            _ => default,
        }
    }

    /// Reads an optional `usize` variable, falling back to `default`.
    fn get_optional_usize(var: &str, default: usize) -> usize {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => match value.parse::<usize>() {
                Ok(value) => value,
                Err(error) => {
                    error!(
                        "Invalid usize value for {}='{}': {}; using default {}",
                        var, value, error, default
                    );
                    default
                }
            },
            _ => default,
        }
    }

    /// Reads an optional `u64` variable, falling back to `default`.
    fn get_optional_number(var: &str, default: u64) -> u64 {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => match value.parse::<u64>() {
                Ok(value) => value,
                Err(error) => {
                    error!(
                        "Invalid number value for {}='{}': {}; using default {}",
                        var, value, error, default
                    );
                    default
                }
            },
            _ => default,
        }
    }
}
