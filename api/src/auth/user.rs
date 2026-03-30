//! Authenticated-user extractor for Axum route handlers.
//!
//! Provides [`AuthUser`], an Axum [`FromRequestParts`] extractor that
//! reads the `access_token` cookie, validates the JWT, and yields the
//! caller's user ID. Including `AuthUser` as a handler parameter is
//! sufficient to enforce authentication on a route.

use axum::{extract::FromRequestParts, http::request::Parts};
use log::error;
use uuid::Uuid;

use crate::auth::jwt::JwtUtil;
use crate::core::error::ApiErrorResponse;
use crate::routes::app::AppState;

/// An authenticated user extracted from an incoming request.
///
/// Add this type to a route handler's parameter list to require a
/// valid `access_token` cookie. The extractor will reject the request
/// with [`ApiErrorResponse::Unauthorized`] if the token is missing or
/// invalid.
pub struct AuthUser {
    /// The unique identifier of the authenticated user.
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiErrorResponse;

    /// Extracts an [`AuthUser`] from the request cookies.
    ///
    /// # Arguments
    ///
    /// * `parts` — The HTTP request head (headers, URI, etc.).
    /// * `state` — Shared application state containing the
    ///   [`Config`](crate::core::config::Config) used for JWT
    ///   validation.
    ///
    /// # Returns
    ///
    /// An [`AuthUser`] populated with the user ID from the JWT
    /// claims.
    ///
    /// # Errors
    ///
    /// * [`ApiErrorResponse::Unauthorized`] — if the `access_token`
    ///   cookie is missing or the JWT is invalid/expired.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookies = parts
            .headers
            .get_all("cookie")
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|s| s.split(';'))
            .map(|s| s.trim())
            .find(|s| s.starts_with("access_token="))
            .ok_or_else(|| ApiErrorResponse::Unauthorized("Missing access token".to_string()))?;

        let token = cookies.strip_prefix("access_token=").unwrap_or("");

        let token_data = JwtUtil::validate_token(token, &state.config).map_err(|error| {
            error!("Failed to validate access token from cookies: {:?}", error);
            ApiErrorResponse::Unauthorized("Invalid or expired token".to_string())
        })?;

        let user_id = token_data.claims.sub;

        Ok(AuthUser { user_id })
    }
}
