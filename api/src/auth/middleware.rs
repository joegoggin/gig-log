use actix_web::{FromRequest, HttpRequest, dev::Payload};
use futures::future::{Ready, ok, err};
use uuid::Uuid;

use crate::auth::jwt::decode_access_token;
use crate::core::error::ApiError;

pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub email: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Extract access token from cookie
        let token = match req.cookie("access_token") {
            Some(cookie) => cookie.value().to_string(),
            None => return err(ApiError::Unauthorized),
        };

        // Get JWT secret from app data
        let env = match req.app_data::<actix_web::web::Data<crate::core::env::Env>>() {
            Some(env) => env,
            None => return err(ApiError::InternalError("Environment not configured".to_string())),
        };

        // Decode and validate token
        match decode_access_token(&token, &env.jwt_secret) {
            Ok(claims) => {
                match Uuid::parse_str(&claims.sub) {
                    Ok(user_id) => ok(AuthenticatedUser {
                        user_id,
                        email: claims.email,
                    }),
                    Err(_) => err(ApiError::TokenInvalid),
                }
            }
            Err(e) => err(e),
        }
    }
}
