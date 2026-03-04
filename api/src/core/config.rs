use std::env;

use anyhow::Error;
use dotenvy::dotenv;

use crate::core::app::AppResult;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_env: String,
    pub web_origin: String,
    pub database_url: String,
    pub auto_apply_migrations: bool,
    pub jwt_secret: String,
    pub jwt_access_token_expiry_seconds: u64,
    pub jwt_refresh_token_expiry_seconds: u64,
    pub resend_api_key: String,
    pub resend_from_email: String,
    pub auth_code_expiry_seconds: u64,
    pub log_level: String,
    pub log_verbose: bool,
    pub log_http_max_body: u64,
}

impl Config {
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
        let log_verbose = Self::get_optional_bool("LOG_VERBOSE", true);
        let log_http_max_body = Self::get_optional_number("LOG_HTTP_MAX_BODY_BYTES", 16384);

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

    pub fn is_development(&self) -> bool {
        Self::is_development_env(&self.app_env)
    }

    pub fn is_development_env(app_env: &str) -> bool {
        let normalized = app_env.trim().to_lowercase();
        normalized == "development" || normalized == "dev"
    }

    fn get_var_from_env(var: &str) -> AppResult<String> {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => Ok(value),
            _ => {
                let error_message = format!("`{}` environment variable not set.", var);

                Err(Error::msg(error_message))
            }
        }
    }

    fn get_optional_string(var: &str, default: impl Into<String>) -> String {
        match Self::get_var_from_env(var) {
            Ok(value) => value,
            Err(_) => default.into(),
        }
    }

    fn get_optional_bool(var: &str, default: bool) -> bool {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => match value.as_str() {
                "true" => true,
                "false" => false,
                _ => default,
            },
            _ => default,
        }
    }

    fn get_optional_number(var: &str, default: u64) -> u64 {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => match value.parse::<u64>() {
                Ok(value) => value,
                Err(_) => default,
            },
            _ => default,
        }
    }
}
