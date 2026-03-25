use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::config::Config;
use crate::core::error::ApiErrorResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtUtil;

impl JwtUtil {
    pub fn generate_access_token(
        user_id: Uuid,
        config: &Config,
    ) -> Result<String, ApiErrorResponse> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            exp: now + config.jwt_access_token_expiry_seconds as i64,
            iat: now,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|error| {
            error!("Failed to generate access token: {}", error);
            ApiErrorResponse::InternalServerError("Failed to generate access token".to_string())
        })
    }

    pub fn generate_refresh_token(
        user_id: Uuid,
        config: &Config,
    ) -> Result<String, ApiErrorResponse> {
        let now = Utc::now().timestamp();
        let claims = Claims {
            sub: user_id,
            exp: now + config.jwt_refresh_token_expiry_seconds as i64,
            iat: now,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_bytes()),
        )
        .map_err(|error| {
            error!("Failed to generate refresh token: {}", error);
            ApiErrorResponse::InternalServerError("Failed to generate refresh token".to_string())
        })
    }

    pub fn validate_token(
        token: &str,
        config: &Config,
    ) -> Result<TokenData<Claims>, ApiErrorResponse> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|error| {
            error!("Failed to validate token: {}", error);
            ApiErrorResponse::BadRequest("Invalid or expired token".to_string())
        })
    }
}
