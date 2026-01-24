use actix_web::cookie::{Cookie, SameSite, time::Duration};

pub fn create_access_token_cookie<'a>(
    token: &'a str,
    domain: &'a str,
    secure: bool,
) -> Cookie<'a> {
    Cookie::build("access_token", token)
        .path("/")
        .domain(domain.to_string())
        .http_only(true)
        .secure(secure)
        .same_site(SameSite::Strict)
        .max_age(Duration::minutes(15))
        .finish()
}

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

pub fn clear_access_token_cookie(domain: &str) -> Cookie<'static> {
    Cookie::build("access_token", "")
        .path("/")
        .domain(domain.to_string())
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO)
        .finish()
}

pub fn clear_refresh_token_cookie(domain: &str) -> Cookie<'static> {
    Cookie::build("refresh_token", "")
        .path("/auth")
        .domain(domain.to_string())
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO)
        .finish()
}
