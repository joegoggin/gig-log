//! JWT claim types and token helpers for authentication.
//!
//! This module creates and validates access/refresh tokens used by the API.
//! Access tokens carry user identity and email, while refresh tokens include
//! a unique token identifier (`jti`) for rotation and revocation workflows.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::ApiError;

/// Claims stored in short-lived access tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    /// User ID as a UUID string.
    pub sub: String,
    /// Authenticated user email.
    pub email: String,
    /// Expiration timestamp (Unix epoch seconds).
    pub exp: usize,
    /// Issued-at timestamp (Unix epoch seconds).
    pub iat: usize,
    /// Token type marker. Expected value: `access`.
    pub token_type: String,
}

/// Claims stored in long-lived refresh tokens.
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    /// User ID as a UUID string.
    pub sub: String,
    /// Expiration timestamp (Unix epoch seconds).
    pub exp: usize,
    /// Issued-at timestamp (Unix epoch seconds).
    pub iat: usize,
    /// Token type marker. Expected value: `refresh`.
    pub token_type: String,
    /// Unique token identifier used for rotation/revocation.
    pub jti: String,
}

/// Creates and signs an access token for a user.
///
/// # Arguments
///
/// - `user_id` - Authenticated user's unique identifier
/// - `email` - Authenticated user's email address
/// - `secret` - JWT signing secret
/// - `expiry_seconds` - Access token lifetime in seconds
///
/// # Errors
///
/// Returns [`ApiError`] if token signing fails.
pub fn create_access_token(
    user_id: Uuid,
    email: &str,
    secret: &str,
    expiry_seconds: u64,
) -> Result<String, ApiError> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiry_seconds as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = AccessTokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        exp,
        iat,
        token_type: "access".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(token)
}

/// Creates and signs a refresh token for a user.
///
/// Returns the signed token and generated `jti`.
///
/// # Arguments
///
/// - `user_id` - Authenticated user's unique identifier
/// - `secret` - JWT signing secret
/// - `expiry_seconds` - Refresh token lifetime in seconds
///
/// # Errors
///
/// Returns [`ApiError`] if token signing fails.
pub fn create_refresh_token(
    user_id: Uuid,
    secret: &str,
    expiry_seconds: u64,
) -> Result<(String, String), ApiError> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(expiry_seconds as i64)).timestamp() as usize;
    let iat = now.timestamp() as usize;
    let jti = Uuid::new_v4().to_string();

    let claims = RefreshTokenClaims {
        sub: user_id.to_string(),
        exp,
        iat,
        token_type: "refresh".to_string(),
        jti: jti.clone(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok((token, jti))
}

/// Decodes and validates an access token.
///
/// Also verifies the custom `token_type` claim is `access`.
///
/// # Arguments
///
/// - `token` - JWT access token string
/// - `secret` - JWT verification secret
///
/// # Errors
///
/// Returns [`ApiError::TokenInvalid`] for wrong token type or invalid token data.
pub fn decode_access_token(token: &str, secret: &str) -> Result<AccessTokenClaims, ApiError> {
    let token_data = decode::<AccessTokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    if token_data.claims.token_type != "access" {
        return Err(ApiError::TokenInvalid);
    }

    Ok(token_data.claims)
}

/// Decodes and validates a refresh token.
///
/// Also verifies the custom `token_type` claim is `refresh`.
///
/// # Arguments
///
/// - `token` - JWT refresh token string
/// - `secret` - JWT verification secret
///
/// # Errors
///
/// Returns [`ApiError::TokenInvalid`] for wrong token type or invalid token data.
pub fn decode_refresh_token(token: &str, secret: &str) -> Result<RefreshTokenClaims, ApiError> {
    let token_data = decode::<RefreshTokenClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;

    if token_data.claims.token_type != "refresh" {
        return Err(ApiError::TokenInvalid);
    }

    Ok(token_data.claims)
}
