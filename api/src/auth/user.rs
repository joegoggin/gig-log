use axum::{extract::FromRequestParts, http::request::Parts};
use log::error;
use uuid::Uuid;

use crate::auth::jwt::JwtUtil;
use crate::core::error::ApiErrorResponse;
use crate::routes::app::AppState;

pub struct AuthUser {
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiErrorResponse;

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
