//! Authentication and account management endpoints.
//!
//! Provides [`AuthController`] with handlers for sign-up, log-in,
//! log-out, token refresh, password management, and email change flows.

use axum::{Json, extract::State};
use axum_extra::extract::CookieJar;
use chrono::{Duration, Utc};
use gig_log_common::models::generic::MessageResponse;
use gig_log_common::models::user::{
    ChangePasswordRequest, ConfirmEmailRequest, ForgotPasswordRequest, LogInRequest,
    RequestEmailChangeRequest, SetPasswordRequest, SignUpRequest, User,
    VerifyForgotPasswordRequest,
};
use log::{error, warn};
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

/// Handlers for authentication and account management routes.
pub struct AuthController;

impl AuthController {
    /// Registers a new user account.
    ///
    /// Mapped to `POST /sign-up`. Creates the user, generates an email
    /// verification code, and sends a confirmation email.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`] providing database and config access.
    /// * `body` — A [`ValidatedJson<SignUpRequest>`] containing the user's
    ///   name, email, and password.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming account creation.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the email is already in use.
    /// Returns [`ApiErrorResponse::InternalServerError`] if password hashing
    /// or email sending fails.
    pub async fn sign_up(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<SignUpRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        match UserRepo::find_user_by_email(&state.db_pool, &body.email).await {
            Ok(_) => {
                return Err(ApiErrorResponse::BadRequest(
                    "Email already in use".to_string(),
                ));
            }
            Err(ApiErrorResponse::NotFound(_)) => {}
            Err(error) => {
                error!(
                    "Failed to check for existing user during sign-up: {:?}",
                    error
                );
                return Err(error);
            }
        }

        let password_hash = PasswordUtil::hash_password(&body.password).map_err(|error| {
            error!("Failed to hash password during sign-up: {:?}", error);
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

    /// Confirms a new user's email address.
    ///
    /// Mapped to `POST /confirm-email`. Validates the verification code
    /// and marks the user's email as confirmed.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<ConfirmEmailRequest>`] containing the
    ///   verification code.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the email was verified.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the code is invalid or
    /// expired.
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
        .map_err(|error| {
            error!("Failed to validate email confirmation code: {:?}", error);
            ApiErrorResponse::BadRequest("Invalid or expired code".to_string())
        })?;

        AuthCodeRepo::mark_used(&state.db_pool, auth_code.id).await?;
        UserRepo::confirm_email(&state.db_pool, auth_code.user_id).await?;

        let response = MessageResponse {
            message: "Email confirmed successfully.".to_string(),
        };

        Ok(Json::from(response))
    }

    /// Authenticates a user and issues session tokens.
    ///
    /// Mapped to `POST /log-in`. Verifies credentials, generates JWT
    /// access and refresh tokens, stores the refresh token hash, and
    /// sets both tokens as HTTP cookies.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `jar` — The [`CookieJar`] to receive the new session cookies.
    /// * `body` — A [`ValidatedJson<LogInRequest>`] containing email and
    ///   password.
    ///
    /// # Returns
    ///
    /// A tuple of the updated [`CookieJar`] and [`Json<User>`] with the
    /// authenticated user's profile.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if credentials are invalid
    /// or the email is not yet confirmed.
    pub async fn log_in(
        state: State<AppState>,
        jar: CookieJar,
        ValidatedJson(body): ValidatedJson<LogInRequest>,
    ) -> ApiResult<(CookieJar, Json<User>)> {
        let user = UserRepo::find_user_by_email(&state.db_pool, &body.email)
            .await
            .map_err(|error| {
                error!("Failed to find user during log-in: {:?}", error);
                ApiErrorResponse::BadRequest("Invalid credentials".to_string())
            })?;

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

    /// Ends the current session and revokes refresh tokens.
    ///
    /// Mapped to `POST /log-out`. Attempts to revoke the specific refresh
    /// token from the cookie. If that fails, falls back to revoking all
    /// refresh tokens for the user identified by the access token. Clears
    /// both session cookies regardless.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `jar` — The [`CookieJar`] containing the session cookies to clear.
    ///
    /// # Returns
    ///
    /// The updated [`CookieJar`] with session cookies removed.
    ///
    /// # Errors
    ///
    /// Returns an [`ApiErrorResponse`] if database operations fail during
    /// token revocation.
    pub async fn log_out(State(state): State<AppState>, jar: CookieJar) -> ApiResult<CookieJar> {
        let mut revoked_by_refresh_cookie = false;

        if let Some(refresh_cookie) = jar.get("refresh_token") {
            let token_hash = Self::sha256_hash(refresh_cookie.value());
            revoked_by_refresh_cookie =
                RefreshTokenRepo::revoke_token(&state.db_pool, &token_hash).await?;

            if !revoked_by_refresh_cookie {
                warn!(
                    "Refresh token cookie was present but no active row was revoked; falling back to user-wide revocation"
                );
            }
        } else {
            warn!(
                "Logout request did not include a refresh_token cookie; falling back to user-wide revocation"
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
                    Err(error) => {
                        warn!(
                            "Could not validate access token for fallback logout revocation: {:?}",
                            error
                        );
                    }
                }
            } else {
                warn!(
                    "Logout request did not include an access_token cookie for fallback revocation"
                );
            }
        }

        let jar = jar
            .add(CookiesUtil::clear_access_cookie())
            .add(CookiesUtil::clear_refresh_cookie());

        Ok(jar)
    }

    /// Rotates the session tokens using the current refresh token.
    ///
    /// Mapped to `POST /refresh`. Validates the existing refresh token,
    /// revokes it, issues a new access/refresh token pair, and updates
    /// the cookies.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `jar` — The [`CookieJar`] containing the current refresh token.
    ///
    /// # Returns
    ///
    /// A tuple of the updated [`CookieJar`] and [`Json<User>`] with the
    /// authenticated user's profile.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the refresh token is
    /// missing, invalid, or does not match a stored record.
    pub async fn refresh(
        State(state): State<AppState>,
        jar: CookieJar,
    ) -> ApiResult<(CookieJar, Json<User>)> {
        let refresh_cookie = jar
            .get("refresh_token")
            .ok_or_else(|| ApiErrorResponse::BadRequest("Missing refresh token".to_string()))?;

        let old_token = refresh_cookie.value();
        let token_data = JwtUtil::validate_token(old_token, &state.config).map_err(|error| {
            error!(
                "Failed to validate refresh token during token refresh: {:?}",
                error
            );
            ApiErrorResponse::BadRequest("Invalid refresh token".to_string())
        })?;
        let old_hash = Self::sha256_hash(old_token);

        let token_record = RefreshTokenRepo::find_by_hash(&state.db_pool, &old_hash)
            .await
            .map_err(|error| {
                error!(
                    "Failed to find refresh token record for refresh flow: {:?}",
                    error
                );
                ApiErrorResponse::BadRequest("Invalid refresh token".to_string())
            })?;

        if token_data.claims.sub != token_record.user_id {
            return Err(ApiErrorResponse::BadRequest(
                "Invalid refresh token".to_string(),
            ));
        }

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

    /// Returns the currently authenticated user's profile.
    ///
    /// Mapped to `GET /me`. Requires a valid access token.
    ///
    /// # Arguments
    ///
    /// * `auth` — The [`AuthUser`] extracted from the access token.
    /// * `state` — The shared [`AppState`].
    ///
    /// # Returns
    ///
    /// A [`Json<User>`] containing the authenticated user's profile.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::NotFound`] if the user no longer exists.
    pub async fn me(auth: AuthUser, State(state): State<AppState>) -> ApiResult<Json<User>> {
        let user = UserRepo::find_user_by_id(&state.db_pool, auth.user_id).await?;

        Ok(Json(user))
    }

    /// Initiates the forgot-password flow.
    ///
    /// Mapped to `POST /forgot-password`. If an account exists for the
    /// given email, generates a password-reset code and sends it via
    /// email. Always returns a success message to avoid leaking whether
    /// the account exists.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<ForgotPasswordRequest>`] containing the
    ///   email address.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] with a generic confirmation message.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`] if the email
    /// fails to send.
    pub async fn forgot_password(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<ForgotPasswordRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        let user = match UserRepo::find_user_by_email(&state.db_pool, &body.email).await {
            Ok(user) => Some(user),
            Err(ApiErrorResponse::NotFound(_)) => None,
            Err(error) => {
                error!(
                    "Failed to look up user during forgot-password flow: {:?}",
                    error
                );
                return Err(error);
            }
        };

        if let Some(user) = user {
            let reset_code = code::generate();
            let expires_at = Utc::now() + Duration::minutes(15);

            AuthCodeRepo::insert_code(
                &state.db_pool,
                user.id,
                &reset_code,
                AuthCodeType::PasswordReset,
                expires_at,
                None,
            )
            .await?;

            let sender = AuthSender::new(
                state.email_client.clone(),
                user.email.clone(),
                reset_code.clone(),
            );

            if let Err(error) = sender.send_reset_password().await {
                error!("Failed to send forgot-password email: {:?}", error);
                return Err(ApiErrorResponse::InternalServerError(
                    "Failed to send email".to_string(),
                ));
            }
        }

        let response = MessageResponse {
            message: "If an account exists for this email, a reset code has been sent.".to_string(),
        };

        Ok(Json(response))
    }

    /// Validates a forgot-password reset code without consuming it.
    ///
    /// Mapped to `POST /verify-forgot-password`. Allows the client to
    /// check whether a reset code is still valid before presenting the
    /// new-password form.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<VerifyForgotPasswordRequest>`]
    ///   containing the reset code.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the code is valid.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the code is invalid or
    /// expired.
    pub async fn verify_forgot_password(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<VerifyForgotPasswordRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        AuthCodeRepo::find_valid_code(&state.db_pool, &body.code, AuthCodeType::PasswordReset)
            .await
            .map_err(|error| {
                error!("Failed to verify forgot-password code: {:?}", error);
                ApiErrorResponse::BadRequest("Invalid or expired reset code".to_string())
            })?;

        Ok(Json(MessageResponse {
            message: "Reset code is valid.".to_string(),
        }))
    }

    /// Resets the user's password using a valid reset code.
    ///
    /// Mapped to `POST /set-password`. Validates the reset code, hashes
    /// the new password, updates the stored hash, marks the code as used,
    /// and revokes all existing refresh tokens for the user.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<SetPasswordRequest>`] containing the
    ///   reset code and new password.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the password was reset.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the reset code is
    /// invalid or expired. Returns
    /// [`ApiErrorResponse::InternalServerError`] if password hashing fails.
    pub async fn set_password(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<SetPasswordRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        let auth_code =
            AuthCodeRepo::find_valid_code(&state.db_pool, &body.code, AuthCodeType::PasswordReset)
                .await
                .map_err(|error| {
                    error!(
                        "Failed to validate reset code during set-password: {:?}",
                        error
                    );
                    ApiErrorResponse::BadRequest("Invalid or expired reset code".to_string())
                })?;

        let password_hash = PasswordUtil::hash_password(&body.new_password).map_err(|error| {
            error!("Failed to hash password during set-password: {:?}", error);
            ApiErrorResponse::InternalServerError("Failed to hash password".to_string())
        })?;

        UserRepo::update_password(&state.db_pool, auth_code.user_id, &password_hash).await?;
        AuthCodeRepo::mark_used(&state.db_pool, auth_code.id).await?;
        RefreshTokenRepo::revoke_all_for_user(&state.db_pool, auth_code.user_id).await?;

        Ok(Json(MessageResponse {
            message: "Password has been reset successfully.".to_string(),
        }))
    }

    /// Sends a verification code for changing the authenticated user's password.
    ///
    /// Mapped to `POST /request-change-password`. Requires authentication.
    /// Generates a password-change code and emails it to the user.
    ///
    /// # Arguments
    ///
    /// * `auth` — The [`AuthUser`] extracted from the access token.
    /// * `state` — The shared [`AppState`].
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the code was sent.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::NotFound`] if the user no longer exists.
    /// Returns [`ApiErrorResponse::InternalServerError`] if email sending
    /// fails.
    pub async fn request_change_password_code(
        auth: AuthUser,
        State(state): State<AppState>,
    ) -> ApiResult<Json<MessageResponse>> {
        let user = UserRepo::find_user_by_id(&state.db_pool, auth.user_id).await?;

        let verification_code = code::generate();
        let expires_at = Utc::now() + Duration::minutes(10);

        AuthCodeRepo::insert_code(
            &state.db_pool,
            user.id,
            &verification_code,
            AuthCodeType::PasswordChange,
            expires_at,
            None,
        )
        .await?;

        let sender = AuthSender::new(
            state.email_client.clone(),
            user.email.clone(),
            verification_code,
        );
        sender.send_password_change().await?;

        Ok(Json(MessageResponse {
            message: "Verification sent to your email".to_string(),
        }))
    }

    /// Changes the authenticated user's password.
    ///
    /// Mapped to `POST /change-password`. Requires authentication.
    /// Verifies the current password, validates the change code, hashes
    /// the new password, and revokes all refresh tokens.
    ///
    /// # Arguments
    ///
    /// * `auth` — The [`AuthUser`] extracted from the access token.
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<ChangePasswordRequest>`] containing
    ///   the current password, verification code, and new password.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the password was changed.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the current password is
    /// incorrect or the verification code is invalid or expired.
    pub async fn change_password(
        auth: AuthUser,
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<ChangePasswordRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        let current_hash = UserRepo::get_password_hash(&state.db_pool, auth.user_id).await?;

        if !PasswordUtil::verify_password(&body.current_password, &current_hash)? {
            return Err(ApiErrorResponse::BadRequest(
                "Current password is incorrect".to_string(),
            ));
        }

        let auth_code =
            AuthCodeRepo::find_valid_code(&state.db_pool, &body.code, AuthCodeType::PasswordChange)
                .await
                .map_err(|error| {
                    error!("Failed to validate password change code: {:?}", error);
                    ApiErrorResponse::BadRequest("Invalid or expired code".to_string())
                })?;

        if auth_code.user_id != auth.user_id {
            return Err(ApiErrorResponse::BadRequest(
                "Invalid or expired code".to_string(),
            ));
        }

        AuthCodeRepo::mark_used(&state.db_pool, auth_code.id).await?;

        let new_hash = PasswordUtil::hash_password(&body.new_password).map_err(|error| {
            error!(
                "Failed to hash password during password change: {:?}",
                error
            );
            ApiErrorResponse::InternalServerError("Failed to hash password".to_string())
        })?;

        UserRepo::update_password(&state.db_pool, auth.user_id, &new_hash).await?;
        RefreshTokenRepo::revoke_all_for_user(&state.db_pool, auth.user_id).await?;

        Ok(Json(MessageResponse {
            message: "Password changed successfully.".to_string(),
        }))
    }

    /// Initiates an email address change for the authenticated user.
    ///
    /// Mapped to `POST /request-email-change`. Requires authentication.
    /// Checks that the new email is not already in use, generates a
    /// verification code, and sends it to the new address.
    ///
    /// # Arguments
    ///
    /// * `auth` — The [`AuthUser`] extracted from the access token.
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<RequestEmailChangeRequest>`] containing
    ///   the new email address.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the code was sent.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the new email is
    /// already in use.
    pub async fn request_email_change(
        auth: AuthUser,
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<RequestEmailChangeRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        match UserRepo::find_user_by_email(&state.db_pool, &body.new_email).await {
            Ok(_) => {
                return Err(ApiErrorResponse::BadRequest(
                    "Email already in use".to_string(),
                ));
            }
            Err(ApiErrorResponse::NotFound(_)) => {}
            Err(error) => {
                error!(
                    "Failed to check existing email during request-email-change: {:?}",
                    error
                );
                return Err(error);
            }
        }

        let change_code = code::generate();
        let expires_at = Utc::now() + Duration::minutes(15);

        AuthCodeRepo::insert_code(
            &state.db_pool,
            auth.user_id,
            &change_code,
            AuthCodeType::EmailChange,
            expires_at,
            Some(&body.new_email),
        )
        .await?;

        let sender = AuthSender::new(
            state.email_client.clone(),
            body.new_email.clone(),
            change_code.clone(),
        );

        sender.send_email_change().await?;

        Ok(Json(MessageResponse {
            message: "Verification code sent to new email.".to_string(),
        }))
    }

    /// Completes the email address change using a verification code.
    ///
    /// Mapped to `POST /confirm-email-change`. Validates the email-change
    /// code, updates the user's email address, and marks the code as used.
    ///
    /// # Arguments
    ///
    /// * `state` — The shared [`AppState`].
    /// * `body` — A [`ValidatedJson<ConfirmEmailRequest>`] containing the
    ///   verification code.
    ///
    /// # Returns
    ///
    /// A [`Json<MessageResponse>`] confirming the email was changed.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::BadRequest`] if the code is invalid or
    /// expired. Returns [`ApiErrorResponse::InternalServerError`] if the
    /// code record is missing the new email value.
    pub async fn confirm_email_change(
        State(state): State<AppState>,
        ValidatedJson(body): ValidatedJson<ConfirmEmailRequest>,
    ) -> ApiResult<Json<MessageResponse>> {
        let auth_code =
            AuthCodeRepo::find_valid_code(&state.db_pool, &body.code, AuthCodeType::EmailChange)
                .await
                .map_err(|error| {
                    error!("Failed to validate email change code: {:?}", error);
                    ApiErrorResponse::BadRequest("Invalid or expired code".to_string())
                })?;

        let new_email = auth_code.new_email.as_ref().ok_or_else(|| {
            ApiErrorResponse::InternalServerError("Email change code missing new email".to_string())
        })?;

        UserRepo::update_email_and_confirm(&state.db_pool, auth_code.user_id, new_email).await?;
        AuthCodeRepo::mark_used(&state.db_pool, auth_code.id).await?;

        Ok(Json(MessageResponse {
            message: "Email changed successfully.".to_string(),
        }))
    }

    /// Computes a hex-encoded SHA-256 hash of the given input.
    ///
    /// Used internally to hash refresh tokens before storage, so that
    /// raw token values are never persisted to the database.
    ///
    /// # Arguments
    ///
    /// * `input` — The string to hash.
    ///
    /// # Returns
    ///
    /// The lowercase hex-encoded SHA-256 digest as a [`String`].
    fn sha256_hash(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
