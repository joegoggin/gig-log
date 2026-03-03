use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use gig_log_common::models::generic::MessageResponse;
use gig_log_common::models::user::{ConfirmEmailRequest, SignUpRequest};

use crate::auth::{code, password::PasswordUtil};
use crate::core::error::{ApiErrorResponse, ApiResult};
use crate::email::senders::auth::AuthSender;
use crate::repo::{
    auth_code::{AuthCodeRepo, AuthCodeType},
    user::UserRepo,
};
use crate::routes::app::AppState;

pub struct AuthController;

impl AuthController {
    pub async fn sign_up(
        State(state): State<AppState>,
        Json(body): Json<SignUpRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        if body.password != body.confirm_password {
            return Err(ApiErrorResponse::BadRequest(
                "Passwords do not match".to_string(),
            ));
        }

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
        Json(body): Json<ConfirmEmailRequest>,
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
}
