use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use gig_log_common::models::generic::MessageResponse;
use gig_log_common::models::user::{ConfirmEmailRequest, LogInRequest, SignUpRequest, User};
use sha2::{Digest, Sha256};

use crate::auth::AuthUser;
use crate::auth::cookies::CookiesUtil;
use crate::auth::jwt::JwtUtil;
use crate::auth::{code, password::PasswordUtil};
use crate::core::error::{ApiErrorResponse, ApiResult};
use crate::email::senders::auth::AuthSender;
use crate::extractors::ValidatedJson;
use crate::repo::refresh_token::RefreshTokenRepo;
use crate::repo::{
    auth_code::{AuthCodeRepo, AuthCodeType},
    user::UserRepo,
};
use crate::routes::app::AppState;

pub struct AuthController;

impl AuthController {
    pub async fn sign_up(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<SignUpRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        let existing = UserRepo::find_user_by_email(&state.db_pool, &body.email).await;
        if existing.is_ok() {
            return Err(ApiErrorResponse::BadRequest(
                "Email already in use".to_string(),
            ));
        }

        let password_hash = PasswordUtil::hash_password(&body.password).map_err(|_| {
            ApiErrorResponse::InternalServerError("Failed to hash password".to_string())
        })?;

        let user = UserRepo::insert_user(
            &state.db_pool,
            &body.first_name,
            &body.last_name,
            &body.email,
            &password_hash,
        )
        .await?;

        let verification_code = code::generate();
        let expires_at = Utc::now() + Duration::minutes(15);
        AuthCodeRepo::insert_code(
            &state.db_pool,
            user.id,
            &verification_code,
            AuthCodeType::EmailVerification,
            expires_at,
            None,
        )
        .await?;

        let sender = AuthSender::new(
            state.email_client.clone(),
            user.email.clone(),
            verification_code.clone(),
        );
        sender.send_email_verification().await?;

        let response = MessageResponse {
            message: "Account created. Please check your email to verify.".to_string(),
        };

        Ok(Json::from(response))
    }

    pub async fn confirm_email(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<ConfirmEmailRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        let auth_code = AuthCodeRepo::find_valid_code(
            &state.db_pool,
            &body.code,
            AuthCodeType::EmailVerification,
        )
        .await
        .map_err(|_| ApiErrorResponse::BadRequest("Invalid or expired code".to_string()))?;

        AuthCodeRepo::mark_used(&state.db_pool, auth_code.id).await?;
        UserRepo::confirm_email(&state.db_pool, auth_code.user_id).await?;

        let response = MessageResponse {
            message: "Email confirmed successfully.".to_string(),
        };

        Ok(Json::from(response))
    }

    pub async fn log_in(
        state: State<AppState>,
        jar: CookieJar,
        ValidatedJson(body): ValidatedJson<LogInRequest>,
    ) -> ApiResult<(CookieJar, Json<User>)> {
        let user = UserRepo::find_user_by_email(&state.db_pool, &body.email)
            .await
            .map_err(|_| ApiErrorResponse::BadRequest("Invalid credentials".to_string()))?;

        if !user.email_confirmed {
            return Err(ApiErrorResponse::BadRequest(
                "Please confirm your email before logging in".to_string(),
            ));
        }

        let password_hash = UserRepo::get_password_hash(&state.db_pool, user.id).await?;
        if !PasswordUtil::verify_password(&body.password, &password_hash)? {
            return Err(ApiErrorResponse::BadRequest(
                "Invalid credentials".to_string(),
            ));
        }

        let access_token = JwtUtil::generate_access_token(user.id, &state.config)?;
        let refresh_token = JwtUtil::generate_refresh_token(user.id, &state.config)?;

        let token_hash = Self::sha256_hash(&refresh_token);
        RefreshTokenRepo::insert_token(&state.db_pool, user.id, &token_hash).await?;

        let jar = jar
            .add(CookiesUtil::build_access_cookie(
                &access_token,
                &state.config,
            ))
            .add(CookiesUtil::build_refresh_cookie(
                &refresh_token,
                &state.config,
            ));

        Ok((jar, Json(user)))
    }

    pub async fn log_out(State(state): State<AppState>, jar: CookieJar) -> ApiResult<CookieJar> {
        let mut revoked_by_refresh_cookie = false;

        if let Some(refresh_cookie) = jar.get("refresh_token") {
            let token_hash = Self::sha256_hash(refresh_cookie.value());
            revoked_by_refresh_cookie =
                RefreshTokenRepo::revoke_token(&state.db_pool, &token_hash).await?;

            if !revoked_by_refresh_cookie {
                println!(
                    "Warning: refresh token cookie was present but no active row was revoked; falling back to user-wide revocation"
                );
            }
        } else {
            println!(
                "Warning: logout request did not include a refresh_token cookie; falling back to user-wide revocation"
            );
        }

        if !revoked_by_refresh_cookie {
            if let Some(access_cookie) = jar.get("access_token") {
                match JwtUtil::validate_token(access_cookie.value(), &state.config) {
                    Ok(token_data) => {
                        RefreshTokenRepo::revoke_all_for_user(
                            &state.db_pool,
                            token_data.claims.sub,
                        )
                        .await?;
                    }
                    Err(_) => {
                        println!(
                            "Warning: could not validate access token for fallback logout revocation"
                        );
                    }
                }
            } else {
                println!(
                    "Warning: logout request did not include an access_token cookie for fallback revocation"
                );
            }
        }

        let jar = jar
            .add(CookiesUtil::clear_access_cookie())
            .add(CookiesUtil::clear_refresh_cookie());

        Ok(jar)
    }

    pub async fn refresh(
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> ApiResult<(CookieJar, Json<User>)> {
        let refresh_cookie = jar
            .get("refresh_token")
            .ok_or_else(|| ApiErrorResponse::BadRequest("Missing refresh token".to_string()))?;

        let old_token = refresh_cookie.value();
        let old_hash = Self::sha256_hash(old_token);

        let token_record = RefreshTokenRepo::find_by_hash(&state.db_pool, &old_hash)
            .await
            .map_err(|_| ApiErrorResponse::BadRequest("Invalid refresh token".to_string()))?;

        RefreshTokenRepo::revoke_token(&state.db_pool, &old_hash).await?;

        let new_access_token = JwtUtil::generate_access_token(token_record.user_id, &state.config)?;
        let new_refresh_token =
            JwtUtil::generate_refresh_token(token_record.user_id, &state.config)?;
        let new_refresh_hash = Self::sha256_hash(&new_refresh_token);

        RefreshTokenRepo::insert_token(&state.db_pool, token_record.user_id, &new_refresh_hash)
            .await?;

        let user = UserRepo::find_user_by_id(&state.db_pool, token_record.user_id).await?;

        let jar = jar
            .add(CookiesUtil::build_access_cookie(
                &new_access_token,
                &state.config,
            ))
            .add(CookiesUtil::build_refresh_cookie(
                &new_refresh_token,
                &state.config,
            ));

        Ok((jar, Json(user)))
    }

    pub async fn me(auth: AuthUser, State(state): State<AppState>) -> ApiResult<Json<User>> {
        let user = UserRepo::find_user_by_id(&state.db_pool, auth.user_id).await?;

        Ok(Json(user))
    }

    fn sha256_hash(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
