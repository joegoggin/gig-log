use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::error::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub email: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub token_type: String,
    pub jti: String,
}

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
