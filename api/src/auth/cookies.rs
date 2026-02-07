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

#[cfg(test)]
mod tests {
    use super::{
        clear_access_token_cookie, clear_refresh_token_cookie, create_access_token_cookie,
        create_refresh_token_cookie,
    };
    use actix_web::cookie::{SameSite, time::Duration};

    #[test]
    // Verifies access-token cookies include expected security and scope attributes.
    fn create_access_token_cookie_sets_expected_attributes() {
        let cookie = create_access_token_cookie("token", "localhost", true);

        assert_eq!(cookie.name(), "access_token");
        assert_eq!(cookie.value(), "token");
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.domain(), Some("localhost"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
        assert_eq!(cookie.max_age(), Some(Duration::minutes(15)));
    }

    #[test]
    // Verifies refresh-token cookies include expected security and path attributes.
    fn create_refresh_token_cookie_sets_expected_attributes() {
        let cookie = create_refresh_token_cookie("refresh", "example.com", false);

        assert_eq!(cookie.name(), "refresh_token");
        assert_eq!(cookie.value(), "refresh");
        assert_eq!(cookie.path(), Some("/auth"));
        assert_eq!(cookie.domain(), Some("example.com"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(false));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
        assert_eq!(cookie.max_age(), Some(Duration::days(7)));
    }

    #[test]
    // Verifies clear-cookie helpers expire auth cookies immediately.
    fn clear_cookie_helpers_expire_tokens() {
        let access = clear_access_token_cookie("localhost");
        let refresh = clear_refresh_token_cookie("localhost");

        assert_eq!(access.value(), "");
        assert_eq!(access.max_age(), Some(Duration::ZERO));
        assert_eq!(refresh.value(), "");
        assert_eq!(refresh.max_age(), Some(Duration::ZERO));
        assert_eq!(refresh.path(), Some("/auth"));
    }
}
