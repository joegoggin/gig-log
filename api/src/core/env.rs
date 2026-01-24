use std::env;

use anyhow::Error;
use dotenvy::dotenv;

use crate::core::app::AppResult;

#[derive(Debug)]
pub struct Env {
    pub database_url: String,
    pub cors_allowed_origin: String,
    pub port: u16,
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

        Ok(Self {
            database_url,
            cors_allowed_origin,
            port,
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
