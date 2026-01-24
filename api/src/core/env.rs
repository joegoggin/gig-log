use std::env;

use anyhow::Error;
use dotenvy::dotenv;

use crate::core::app::AppResult;

#[derive(Debug, Clone)]
pub struct Env {
    pub database_url: String,
    pub cors_allowed_origin: String,
    pub port: u16,

    // JWT Configuration
    pub jwt_secret: String,
    pub jwt_access_token_expiry_seconds: u64,
    pub jwt_refresh_token_expiry_seconds: u64,

    // Resend Email Service
    pub resend_api_key: String,
    pub resend_from_email: String,

    // Auth Codes
    pub auth_code_expiry_seconds: u64,

    // Cookie Configuration
    pub cookie_domain: String,
    pub cookie_secure: bool,
}

impl Env {
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

    fn get_required_var(var: &str) -> AppResult<String> {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => Ok(value),
            _ => {
                let error_message = format!("`{}` environment variable not set.", var);

                Err(Error::msg(error_message))
            }
        }
    }

    fn get_optional_var(var: &str) -> Option<String> {
        match env::var(var) {
            Ok(value) if !value.trim().is_empty() => Some(value),
            _ => None,
        }
    }
}
