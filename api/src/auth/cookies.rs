use axum_extra::extract::cookie::{Cookie, SameSite};
use time::Duration;

use crate::core::config::Config;

const ACCESS_TOKEN_COOKIE: &str = "access_token";
const REFRESH_TOKEN_COOKIE: &str = "refresh_token";

pub struct CookiesUtil;

impl CookiesUtil {
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

    pub fn clear_access_cookie() -> Cookie<'static> {
        Cookie::build((ACCESS_TOKEN_COOKIE, ""))
            .path("/")
            .max_age(Duration::ZERO)
            .build()
    }

    pub fn clear_refresh_cookie() -> Cookie<'static> {
        Cookie::build((REFRESH_TOKEN_COOKIE, ""))
            .path("/")
            .max_age(Duration::ZERO)
            .build()
    }
}
