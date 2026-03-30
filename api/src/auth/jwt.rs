//! JWT token creation and validation.
//!
//! Provides [`JwtUtil`] for generating and validating access and
//! refresh tokens, and the [`Claims`] payload embedded in each token.

use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::config::Config;
use crate::core::error::ApiErrorResponse;

/// The payload embedded in every JWT issued by the application.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — the authenticated user's ID.
    pub sub: Uuid,
    /// Expiration time as a Unix timestamp (seconds).
    pub exp: i64,
    /// Issued-at time as a Unix timestamp (seconds).
    pub iat: i64,
}

/// Utility for generating and validating JWT tokens.
///
/// Tokens are signed with the HMAC secret stored in [`Config::jwt_secret`].
pub struct JwtUtil;

impl JwtUtil {
    /// Generates a short-lived JWT access token for the given user.
    ///
    /// # Arguments
    ///
    /// * `user_id` — The [`Uuid`] of the authenticated user.
    /// * `config` — Application configuration providing the JWT secret
    ///   and access-token expiry duration.
    ///
    /// # Returns
    ///
    /// The encoded JWT [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`] if token
    /// encoding fails.
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

    /// Generates a long-lived JWT refresh token for the given user.
    ///
    /// # Arguments
    ///
    /// * `user_id` — The [`Uuid`] of the authenticated user.
    /// * `config` — Application configuration providing the JWT secret
    ///   and refresh-token expiry duration.
    ///
    /// # Returns
    ///
    /// The encoded JWT [`String`].
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`] if token
    /// encoding fails.
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

    /// Validates and decodes a JWT token.
    ///
    /// # Arguments
    ///
    /// * `token` — The raw JWT string to validate.
    /// * `config` — Application configuration providing the JWT secret.
    ///
    /// # Returns
    ///
    /// A [`TokenData<Claims>`] containing the decoded [`Claims`] and
    /// token header metadata.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the token is
    /// malformed, expired, or has an invalid signature.
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
