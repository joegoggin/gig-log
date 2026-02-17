//! JWT claim types and token helpers for authentication.
//!
//! This module creates and validates access/refresh tokens used by the API.
//! Access tokens carry user identity and email, while refresh tokens include
//! a unique token identifier (`jti`) for rotation and revocation workflows.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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
    /// Whether the session should persist across browser restarts.
    ///
    /// Defaults to `false` when decoding older tokens that predate
    /// the remember-me claim.
    #[serde(default)]
    pub remember_me: bool,
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
    remember_me: bool,
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
        remember_me,
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{
        create_access_token, create_refresh_token, decode_access_token, decode_refresh_token,
    };
    use crate::core::error::ApiError;

    const TEST_SECRET: &str = "test-secret-for-jwt-unit-tests";

    #[test]
    // Verifies access tokens can be created and decoded with expected claims.
    fn access_token_round_trip_succeeds() {
        let user_id = Uuid::new_v4();
        let email = "user@example.com";

        let token =
            create_access_token(user_id, email, TEST_SECRET, 900).expect("access token created");
        let claims = decode_access_token(&token, TEST_SECRET).expect("token should decode");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.email, email);
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    // Verifies refresh tokens can be created and decoded with expected claims.
    fn refresh_token_round_trip_succeeds() {
        let user_id = Uuid::new_v4();

        let (token, jti) = create_refresh_token(user_id, TEST_SECRET, 60 * 60, true)
            .expect("refresh token created");
        let claims = decode_refresh_token(&token, TEST_SECRET).expect("token should decode");

        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.jti, jti);
        assert_eq!(claims.token_type, "refresh");
        assert!(claims.remember_me);
    }

    #[test]
    // Verifies the access decoder rejects refresh-token payloads.
    fn decode_access_token_rejects_refresh_token_type() {
        let user_id = Uuid::new_v4();
        let (refresh_token, _) = create_refresh_token(user_id, TEST_SECRET, 60 * 60, false)
            .expect("refresh token created");

        let result = decode_access_token(&refresh_token, TEST_SECRET);

        assert!(matches!(result, Err(ApiError::TokenInvalid)));
    }

    #[test]
    // Verifies the refresh decoder rejects access-token payloads.
    fn decode_refresh_token_rejects_access_token_type() {
        let user_id = Uuid::new_v4();
        let access_token = create_access_token(user_id, "user@example.com", TEST_SECRET, 900)
            .expect("access token created");

        let result = decode_refresh_token(&access_token, TEST_SECRET);

        assert!(matches!(result, Err(ApiError::TokenInvalid)));
    }
}
