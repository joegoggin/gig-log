use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use gig_log_common::models::user::SignUpRequest;

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
    ) -> ApiResult<Json<serde_json::Value>> {
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

        Ok(Json(serde_json::json!({
            "message": "Account created. Please check your email to verify."
        })))
    }
}
