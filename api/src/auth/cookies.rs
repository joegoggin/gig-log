//! HTTP cookie helpers for authentication tokens.
//!
//! This module centralizes secure cookie configuration for access and refresh
//! tokens so handlers can set and clear auth cookies consistently.

use actix_web::cookie::{Cookie, SameSite, time::Duration};

/// Builds the `access_token` cookie for authenticated requests.
///
/// # Arguments
///
/// - `token` - Signed JWT access token
/// - `domain` - Cookie domain (for example `localhost` or production domain)
/// - `secure` - Whether to mark the cookie as `Secure`
pub fn create_access_token_cookie<'a>(token: &'a str, domain: &'a str, secure: bool) -> Cookie<'a> {
    Cookie::build("access_token", token)
        .path("/")
        .domain(domain.to_string())
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict)
        .max_age(Duration::minutes(15))
        .finish()
}

/// Builds the `refresh_token` cookie used by refresh/logout endpoints.
///
/// # Arguments
///
/// - `token` - Signed JWT refresh token
/// - `domain` - Cookie domain (for example `localhost` or production domain)
/// - `secure` - Whether to mark the cookie as `Secure`
pub fn create_refresh_token_cookie<'a>(
    token: &'a str,
    domain: &'a str,
    secure: bool,
) -> Cookie<'a> {
    Cookie::build("refresh_token", token)
        .path("/auth")
        .domain(domain.to_string())
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict)
        .max_age(Duration::days(7))
        .finish()
}

/// Builds an expired `access_token` cookie to clear the browser value.
///
/// # Arguments
///
/// - `domain` - Cookie domain used when the token cookie was originally set
pub fn clear_access_token_cookie(domain: &str) -> Cookie<'static> {
    Cookie::build("access_token", "")
        .path("/")
        .domain(domain.to_string())
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO)
        .finish()
}

/// Builds an expired `refresh_token` cookie to clear the browser value.
///
/// # Arguments
///
/// - `domain` - Cookie domain used when the token cookie was originally set
pub fn clear_refresh_token_cookie(domain: &str) -> Cookie<'static> {
    Cookie::build("refresh_token", "")
        .path("/auth")
        .domain(domain.to_string())
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO)
        .finish()
}
