//! Cookie management for authentication tokens.
//!
//! Provides [`CookiesUtil`] with helpers for building and clearing
//! the HTTP cookies that carry access and refresh JWT tokens.

use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

use crate::core::config::Config;

/// Cookie name for the short-lived access token.
const ACCESS_TOKEN_COOKIE: &str = "access_token";

/// Cookie name for the long-lived refresh token.
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

/// Utility for constructing and clearing authentication cookies.
///
/// All cookies are scoped to the root path (`/`) and marked
/// `HttpOnly` with `SameSite::Lax`. The `Secure` flag is set
/// automatically in non-development environments.
pub struct CookiesUtil;

impl CookiesUtil {
    /// Builds an access-token cookie.
    ///
    /// # Arguments
    ///
    /// * `token` — The signed JWT access token string.
    /// * `config` — Application configuration, used for the token
    ///   expiry and to determine whether the `Secure` flag is set.
    ///
    /// # Returns
    ///
    /// A [`Cookie`] ready to be attached to a response.
    pub fn build_access_cookie(token: &str, config: &Config) -> Cookie<'static> {
        let mut cookie = Cookie::build((ACCESS_TOKEN_COOKIE, token.to_string()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::seconds(
                config.jwt_access_token_expiry_seconds as i64,
            ))
            .build();

        cookie.set_secure(!config.is_development());
        cookie
    }

    /// Builds a refresh-token cookie.
    ///
    /// # Arguments
    ///
    /// * `token` — The signed JWT refresh token string.
    /// * `config` — Application configuration, used for the token
    ///   expiry and to determine whether the `Secure` flag is set.
    ///
    /// # Returns
    ///
    /// A [`Cookie`] ready to be attached to a response.
    pub fn build_refresh_cookie(token: &str, config: &Config) -> Cookie<'static> {
        let mut cookie = Cookie::build((REFRESH_TOKEN_COOKIE, token.to_string()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .max_age(Duration::seconds(
                config.jwt_refresh_token_expiry_seconds as i64,
            ))
            .build();

        cookie.set_secure(!config.is_development());
        cookie
    }

    /// Builds a cookie that clears the access token.
    ///
    /// The cookie has an empty value and a zero max-age, which
    /// instructs the browser to remove it.
    ///
    /// # Returns
    ///
    /// A [`Cookie`] that expires the access token cookie.
    pub fn clear_access_cookie() -> Cookie<'static> {
        Cookie::build((ACCESS_TOKEN_COOKIE, ""))
            .path("/")
            .max_age(Duration::ZERO)
            .build()
    }

    /// Builds a cookie that clears the refresh token.
    ///
    /// The cookie has an empty value and a zero max-age, which
    /// instructs the browser to remove it.
    ///
    /// # Returns
    ///
    /// A [`Cookie`] that expires the refresh token cookie.
    pub fn clear_refresh_cookie() -> Cookie<'static> {
        Cookie::build((REFRESH_TOKEN_COOKIE, ""))
            .path("/")
            .max_age(Duration::ZERO)
            .build()
    }
}
