//! HTTP cookie helpers for authentication tokens.
//!
//! This module centralizes secure cookie configuration for access and refresh
//! tokens so handlers can set and clear auth cookies consistently.

use actix_web::cookie::{time::Duration, Cookie, SameSite};

/// Builds the `access_token` cookie for authenticated requests.
///
/// # Arguments
///
/// - `token` - Signed JWT access token
/// - `domain` - Optional cookie domain (for example `localhost` or production domain)
/// - `secure` - Whether to mark the cookie as `Secure`
/// - `max_age_seconds` - Cookie lifetime in seconds
pub fn create_access_token_cookie<'a>(
    token: &'a str,
    domain: Option<&'a str>,
    secure: bool,
    max_age_seconds: u64,
) -> Cookie<'a> {
    let mut cookie = Cookie::build("access_token", token)
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict)
        .max_age(Duration::seconds(max_age_seconds as i64));

    if let Some(domain) = domain.filter(|domain| !domain.trim().is_empty()) {
        cookie = cookie.domain(domain.to_string());
    }

    cookie.finish()
}

/// Builds the `refresh_token` cookie used by refresh/logout endpoints.
///
/// # Arguments
///
/// - `token` - Signed JWT refresh token
/// - `domain` - Optional cookie domain (for example `localhost` or production domain)
/// - `secure` - Whether to mark the cookie as `Secure`
/// - `max_age_seconds` - Cookie lifetime in seconds
pub fn create_refresh_token_cookie<'a>(
    token: &'a str,
    domain: Option<&'a str>,
    secure: bool,
    max_age_seconds: Option<u64>,
) -> Cookie<'a> {
    let mut cookie = Cookie::build("refresh_token", token)
        .path("/auth")
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict);

    if let Some(max_age_seconds) = max_age_seconds {
        cookie = cookie.max_age(Duration::seconds(max_age_seconds as i64));
    }

    if let Some(domain) = domain.filter(|domain| !domain.trim().is_empty()) {
        cookie = cookie.domain(domain.to_string());
    }

    cookie.finish()
}

/// Builds an expired `access_token` cookie to clear the browser value.
///
/// # Arguments
///
/// - `domain` - Optional cookie domain used when the token cookie was originally set
pub fn clear_access_token_cookie(domain: Option<&str>) -> Cookie<'static> {
    let mut cookie = Cookie::build("access_token", "")
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO);

    if let Some(domain) = domain.filter(|domain| !domain.trim().is_empty()) {
        cookie = cookie.domain(domain.to_string());
    }

    cookie.finish()
}

/// Builds an expired `refresh_token` cookie to clear the browser value.
///
/// # Arguments
///
/// - `domain` - Optional cookie domain used when the token cookie was originally set
pub fn clear_refresh_token_cookie(domain: Option<&str>) -> Cookie<'static> {
    let mut cookie = Cookie::build("refresh_token", "")
        .path("/auth")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO);

    if let Some(domain) = domain.filter(|domain| !domain.trim().is_empty()) {
        cookie = cookie.domain(domain.to_string());
    }

    cookie.finish()
}

#[cfg(test)]
mod tests {
    use super::{
        clear_access_token_cookie, clear_refresh_token_cookie, create_access_token_cookie,
        create_refresh_token_cookie,
    };
    use actix_web::cookie::{time::Duration, SameSite};

    #[test]
    // Verifies access-token cookies include expected security and scope attributes.
    fn create_access_token_cookie_sets_expected_attributes() {
        let cookie = create_access_token_cookie("token", Some("localhost"), true, 900);

        assert_eq!(cookie.name(), "access_token");
        assert_eq!(cookie.value(), "token");
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.domain(), Some("localhost"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
        assert_eq!(cookie.max_age(), Some(Duration::seconds(900)));
    }

    #[test]
    // Verifies refresh-token cookies include expected security and path attributes.
    fn create_refresh_token_cookie_sets_expected_attributes() {
        let cookie =
            create_refresh_token_cookie("refresh", Some("example.com"), false, Some(604_800));

        assert_eq!(cookie.name(), "refresh_token");
        assert_eq!(cookie.value(), "refresh");
        assert_eq!(cookie.path(), Some("/auth"));
        assert_eq!(cookie.domain(), Some("example.com"));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(false));
        assert_eq!(cookie.same_site(), Some(SameSite::Strict));
        assert_eq!(cookie.max_age(), Some(Duration::seconds(604_800)));
    }

    #[test]
    // Verifies clear-cookie helpers expire auth cookies immediately.
    fn clear_cookie_helpers_expire_tokens() {
        let access = clear_access_token_cookie(Some("localhost"));
        let refresh = clear_refresh_token_cookie(Some("localhost"));

        assert_eq!(access.value(), "");
        assert_eq!(access.max_age(), Some(Duration::ZERO));
        assert_eq!(refresh.value(), "");
        assert_eq!(refresh.max_age(), Some(Duration::ZERO));
        assert_eq!(refresh.path(), Some("/auth"));
    }

    #[test]
    // Verifies cookies can be host-only when no domain is configured.
    fn cookie_helpers_support_host_only_mode() {
        let access = create_access_token_cookie("token", None, false, 900);
        let refresh = create_refresh_token_cookie("refresh", None, false, Some(604_800));
        let clear_access = clear_access_token_cookie(None);
        let clear_refresh = clear_refresh_token_cookie(None);

        assert_eq!(access.domain(), None);
        assert_eq!(refresh.domain(), None);
        assert_eq!(clear_access.domain(), None);
        assert_eq!(clear_refresh.domain(), None);
    }

    #[test]
    // Verifies refresh cookies can be session-scoped when no max age is provided.
    fn create_refresh_token_cookie_supports_session_cookie() {
        let cookie = create_refresh_token_cookie("refresh", Some("localhost"), false, None);

        assert_eq!(cookie.max_age(), None);
    }
}
