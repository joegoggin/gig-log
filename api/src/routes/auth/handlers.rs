//! HTTP handler functions for authentication endpoints.
//!
//! This module contains all the handler functions that process authentication
//! requests including user registration, login, logout, email confirmation,
//! and password management.

use actix_web::{HttpRequest, HttpResponse, get, post, web};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};

use crate::auth::codes::{generate_auth_code, hash_code, verify_code};
use crate::auth::cookies::{
    clear_access_token_cookie, clear_refresh_token_cookie, create_access_token_cookie,
    create_refresh_token_cookie,
};
use crate::auth::jwt::{create_access_token, create_refresh_token, decode_refresh_token};
use crate::auth::middleware::AuthenticatedUser;
use crate::auth::password::{hash_password, verify_password};
use crate::core::app_state::AppState;
use crate::core::error::{ApiError, ApiResult};
use crate::extractors::ValidatedJson;
use crate::models::auth_code::AuthCodeType;
use crate::repository::auth::AuthRepo;

use super::payloads::{
    ConfirmEmailRequest, ConfirmEmailResponse, CurrentUserResponse, ForgotPasswordRequest,
    ForgotPasswordResponse, LogInRequest, LogInResponse, LogOutResponse, SetPasswordRequest,
    SetPasswordResponse, SignUpRequest, SignUpResponse, VerifyForgotPasswordRequest,
    VerifyForgotPasswordResponse,
};

/// Registers a new user account.
///
/// Creates a new user with the provided credentials, generates an email
/// confirmation code, and sends it to the user's email address.
///
/// # Route
///
/// `POST /auth/sign-up`
///
/// # Request Body ([`SignUpRequest`])
///
/// - `first_name` - User's first name
/// - `last_name` - User's last name
/// - `email` - User's email address (must be unique)
/// - `password` - User's chosen password (minimum 8 characters)
/// - `confirm` - Password confirmation (must match `password`)
///
/// # Response Body ([`SignUpResponse`])
///
/// - `message` - Success message instructing user to check email
/// - `user_id` - The newly created user's unique identifier
///
/// # Errors
///
/// - `EmailAlreadyExists` - If the email is already registered
/// - `InternalError` - If password hashing or database operations fail
#[post("/auth/sign-up")]
pub async fn sign_up(
    state: web::Data<AppState>,
    body: ValidatedJson<SignUpRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    // Check if email already exists
    if AuthRepo::check_email_exists(&state.pool, &body.email).await? {
        return Err(ApiError::EmailAlreadyExists);
    }

    // Hash password
    let hashed_password = hash_password(&body.password)?;

    // Create user
    let user_id = AuthRepo::create_user(
        &state.pool,
        &body.first_name,
        &body.last_name,
        &body.email,
        &hashed_password,
    )
    .await?;

    // Generate and store auth code
    let code = generate_auth_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + Duration::seconds(state.env.auth_code_expiry_seconds as i64);

    AuthRepo::create_auth_code(
        &state.pool,
        user_id,
        &code_hash,
        AuthCodeType::EmailConfirmation,
        expires_at,
    )
    .await?;

    // Send confirmation email
    state
        .email_sender
        .send_confirmation_email(&body.email, &body.first_name, &code)
        .await?;

    Ok(HttpResponse::Created().json(SignUpResponse {
        message: "Account created. Please check your email for a confirmation code.".to_string(),
        user_id,
    }))
}

/// Confirms a user's email address using the provided auth code.
///
/// Validates the auth code against the stored hash and marks the user's
/// email as confirmed if valid.
///
/// # Route
///
/// `POST /auth/confirm-email`
///
/// # Request Body ([`ConfirmEmailRequest`])
///
/// - `email` - The email address to confirm
/// - `auth_code` - The confirmation code sent to the user's email
///
/// # Response Body ([`ConfirmEmailResponse`])
///
/// - `message` - Confirmation status message
///
/// # Errors
///
/// - `InvalidCredentials` - If no user exists with the given email
/// - `AuthCodeExpired` - If no valid auth code exists
/// - `InvalidAuthCode` - If the provided code doesn't match
#[post("/auth/confirm-email")]
pub async fn confirm_email(
    state: web::Data<AppState>,
    body: ValidatedJson<ConfirmEmailRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    // Find user by email
    let user = AuthRepo::find_user_for_confirmation(&state.pool, &body.email)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    if user.email_confirmed {
        return Ok(HttpResponse::Ok().json(ConfirmEmailResponse {
            message: "Email already confirmed.".to_string(),
        }));
    }

    // Find valid auth code
    let auth_code =
        AuthRepo::find_valid_auth_code(&state.pool, user.id, AuthCodeType::EmailConfirmation)
            .await?
            .ok_or(ApiError::AuthCodeExpired)?;

    // Verify code
    if !verify_code(&body.auth_code, &auth_code.code_hash) {
        return Err(ApiError::InvalidAuthCode);
    }

    // Mark code as used and confirm email in a transaction
    let mut tx = state.pool.begin().await?;

    AuthRepo::mark_auth_code_used(&mut tx, auth_code.id).await?;
    AuthRepo::confirm_user_email(&mut tx, user.id).await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ConfirmEmailResponse {
        message: "Email confirmed successfully.".to_string(),
    }))
}

/// Authenticates a user and issues JWT tokens.
///
/// Verifies the user's credentials, creates access and refresh tokens,
/// stores the refresh token hash in the database, and sets HTTP-only cookies.
///
/// # Route
///
/// `POST /auth/log-in`
///
/// # Request Body ([`LogInRequest`])
///
/// - `email` - User's email address
/// - `password` - User's password
///
/// # Response Body ([`LogInResponse`])
///
/// - `message` - Success message
/// - `user_id` - The authenticated user's unique identifier
///
/// # Errors
///
/// - `InvalidCredentials` - If email doesn't exist or password is incorrect
/// - `EmailNotConfirmed` - If the user hasn't confirmed their email
#[post("/auth/log-in")]
pub async fn log_in(
    state: web::Data<AppState>,
    body: ValidatedJson<LogInRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    // Find user by email
    let user = AuthRepo::find_user_for_login(&state.pool, &body.email)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    // Verify password
    if !verify_password(&body.password, &user.hashed_password)? {
        return Err(ApiError::InvalidCredentials);
    }

    // Check if email is confirmed
    if !user.email_confirmed {
        return Err(ApiError::EmailNotConfirmed);
    }

    // Create access token
    let access_token = create_access_token(
        user.id,
        &user.email,
        &state.env.jwt_secret,
        state.env.jwt_access_token_expiry_seconds,
    )?;

    // Create refresh token
    let (refresh_token, jti) = create_refresh_token(
        user.id,
        &state.env.jwt_secret,
        state.env.jwt_refresh_token_expiry_seconds,
    )?;

    // Hash and store refresh token
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(jti.as_bytes());
        hex::encode(hasher.finalize())
    };
    let expires_at =
        Utc::now() + Duration::seconds(state.env.jwt_refresh_token_expiry_seconds as i64);

    AuthRepo::create_refresh_token(&state.pool, user.id, &token_hash, expires_at).await?;

    // Create cookies
    let access_cookie = create_access_token_cookie(
        &access_token,
        &state.env.cookie_domain,
        state.env.cookie_secure,
        state.env.jwt_access_token_expiry_seconds,
    );
    let refresh_cookie = create_refresh_token_cookie(
        &refresh_token,
        &state.env.cookie_domain,
        state.env.cookie_secure,
        state.env.jwt_refresh_token_expiry_seconds,
    );

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(LogInResponse {
            message: "Logged in successfully.".to_string(),
            user_id: user.id,
        }))
}

/// Logs out the current user by revoking tokens and clearing cookies.
///
/// Attempts to revoke the refresh token if present and valid, then clears
/// both access and refresh token cookies. Always succeeds even if no valid
/// tokens are present.
///
/// # Route
///
/// `POST /auth/log-out`
///
/// # Response Body ([`LogOutResponse`])
///
/// - `message` - Success message
#[post("/auth/log-out")]
pub async fn log_out(
    state: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> ApiResult<HttpResponse> {
    // Try to revoke refresh token if present
    if let Some(refresh_cookie) = req.cookie("refresh_token")
        && let Ok(claims) =
            crate::auth::jwt::decode_refresh_token(refresh_cookie.value(), &state.env.jwt_secret)
    {
        let token_hash = {
            let mut hasher = Sha256::new();
            hasher.update(claims.jti.as_bytes());
            hex::encode(hasher.finalize())
        };

        let _ = AuthRepo::revoke_refresh_token(&state.pool, &token_hash).await;
    }

    // Clear cookies
    let clear_access = clear_access_token_cookie(&state.env.cookie_domain);
    let clear_refresh = clear_refresh_token_cookie(&state.env.cookie_domain);

    Ok(HttpResponse::Ok()
        .cookie(clear_access)
        .cookie(clear_refresh)
        .json(LogOutResponse {
            message: "Logged out successfully.".to_string(),
        }))
}

/// Retrieves the currently authenticated user's profile.
///
/// Requires a valid access token. Returns the user's basic profile information.
///
/// # Route
///
/// `GET /auth/me`
///
/// # Response Body ([`CurrentUserResponse`])
///
/// - `user` - The current user's profile data
///   - `id` - User's unique identifier
///   - `first_name` - User's first name
///   - `last_name` - User's last name
///   - `email` - User's email address
///   - `email_confirmed` - Whether the email has been confirmed
///   - `created_at` - Account creation timestamp
///   - `updated_at` - Last update timestamp
///
/// # Errors
///
/// - `Unauthorized` - If the access token is invalid or the user doesn't exist
#[get("/auth/me")]
pub async fn current_user(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> ApiResult<HttpResponse> {
    let user = AuthRepo::find_user_by_id(&state.pool, auth_user.user_id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    Ok(HttpResponse::Ok().json(CurrentUserResponse { user }))
}

/// Initiates the password reset flow.
///
/// Generates a password reset code and sends it to the user's email.
/// Always returns success to prevent email enumeration attacks, even if
/// the email doesn't exist in the system.
///
/// # Route
///
/// `POST /auth/forgot-password`
///
/// # Request Body ([`ForgotPasswordRequest`])
///
/// - `email` - Email address of the account to reset
///
/// # Response Body ([`ForgotPasswordResponse`])
///
/// - `message` - Generic message (same whether email exists or not for security)
#[post("/auth/forgot-password")]
pub async fn forgot_password(
    state: web::Data<AppState>,
    body: ValidatedJson<ForgotPasswordRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    // Always return success to prevent email enumeration
    let response = ForgotPasswordResponse {
        message: "If an account with this email exists, a password reset code has been sent."
            .to_string(),
    };

    // Find user by email
    let user = match AuthRepo::find_user_for_password_reset(&state.pool, &body.email).await? {
        Some(user) => user,
        None => return Ok(HttpResponse::Ok().json(response)),
    };

    // Invalidate any existing password reset codes
    AuthRepo::invalidate_password_reset_codes(&state.pool, user.id).await?;

    // Generate and store new auth code
    let code = generate_auth_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + Duration::seconds(state.env.auth_code_expiry_seconds as i64);

    AuthRepo::create_auth_code(
        &state.pool,
        user.id,
        &code_hash,
        AuthCodeType::PasswordReset,
        expires_at,
    )
    .await?;

    // Send password reset email
    let _ = state
        .email_sender
        .send_password_reset_email(&body.email, &user.first_name, &code)
        .await;

    Ok(HttpResponse::Ok().json(response))
}

/// Verifies a password reset code and issues tokens.
///
/// Validates the reset code, marks it as used, and issues access/refresh
/// tokens so the user can set a new password.
///
/// # Route
///
/// `POST /auth/verify-forgot-password`
///
/// # Request Body ([`VerifyForgotPasswordRequest`])
///
/// - `email` - Email address of the account
/// - `auth_code` - The password reset code sent to the user's email
///
/// # Response Body ([`VerifyForgotPasswordResponse`])
///
/// - `message` - Success message
///
/// # Errors
///
/// - `InvalidCredentials` - If the email doesn't exist
/// - `AuthCodeExpired` - If no valid reset code exists
/// - `InvalidAuthCode` - If the provided code doesn't match
#[post("/auth/verify-forgot-password")]
pub async fn verify_forgot_password(
    state: web::Data<AppState>,
    body: ValidatedJson<VerifyForgotPasswordRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    // Find user by email
    let user = AuthRepo::find_user_for_verification(&state.pool, &body.email)
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

    // Find valid auth code
    let auth_code =
        AuthRepo::find_valid_auth_code(&state.pool, user.id, AuthCodeType::PasswordReset)
            .await?
            .ok_or(ApiError::AuthCodeExpired)?;

    // Verify code
    if !verify_code(&body.auth_code, &auth_code.code_hash) {
        return Err(ApiError::InvalidAuthCode);
    }

    // Mark code as used
    AuthRepo::mark_auth_code_used_without_tx(&state.pool, auth_code.id).await?;

    // Issue tokens to allow password reset
    let access_token = create_access_token(
        user.id,
        &user.email,
        &state.env.jwt_secret,
        state.env.jwt_access_token_expiry_seconds,
    )?;

    let (refresh_token, jti) = create_refresh_token(
        user.id,
        &state.env.jwt_secret,
        state.env.jwt_refresh_token_expiry_seconds,
    )?;

    // Store refresh token
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(jti.as_bytes());
        hex::encode(hasher.finalize())
    };
    let expires_at =
        Utc::now() + Duration::seconds(state.env.jwt_refresh_token_expiry_seconds as i64);

    AuthRepo::create_refresh_token(&state.pool, user.id, &token_hash, expires_at).await?;

    // Create cookies
    let access_cookie = create_access_token_cookie(
        &access_token,
        &state.env.cookie_domain,
        state.env.cookie_secure,
        state.env.jwt_access_token_expiry_seconds,
    );
    let refresh_cookie = create_refresh_token_cookie(
        &refresh_token,
        &state.env.cookie_domain,
        state.env.cookie_secure,
        state.env.jwt_refresh_token_expiry_seconds,
    );

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(VerifyForgotPasswordResponse {
            message: "Code verified. You can now set a new password.".to_string(),
        }))
}

/// Sets a new password for the authenticated user.
///
/// Updates the user's password, revokes all existing refresh tokens for
/// security, and issues new access/refresh tokens. Requires both a valid
/// access token and an active refresh-session token.
///
/// # Route
///
/// `POST /auth/set-password`
///
/// # Request Body ([`SetPasswordRequest`])
///
/// - `password` - The new password (minimum 8 characters)
/// - `confirm` - Password confirmation (must match `password`)
///
/// # Response Body ([`SetPasswordResponse`])
///
/// - `message` - Success message
///
/// # Errors
///
/// - `Unauthorized` - If not authenticated, no active refresh session exists, or the user no longer exists
/// - `TokenInvalid` - If the refresh token is malformed or has an invalid token type
/// - `TokenExpired` - If the refresh token is expired
/// - `InternalError` - If password hashing or database operations fail
#[post("/auth/set-password")]
pub async fn set_password(
    req: HttpRequest,
    user: AuthenticatedUser,
    state: web::Data<AppState>,
    body: ValidatedJson<SetPasswordRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    // Require an active refresh-session token so logout immediately invalidates set-password access.
    let refresh_cookie = req.cookie("refresh_token").ok_or(ApiError::Unauthorized)?;
    let refresh_claims = decode_refresh_token(refresh_cookie.value(), &state.env.jwt_secret)?;

    if refresh_claims.sub != user.user_id.to_string() {
        return Err(ApiError::Unauthorized);
    }

    let refresh_token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(refresh_claims.jti.as_bytes());
        hex::encode(hasher.finalize())
    };

    // Start transaction early so refresh-session validation and password update
    // are performed atomically against concurrent logout requests.
    let mut tx = state.pool.begin().await?;

    if !AuthRepo::consume_active_refresh_token(&mut tx, user.user_id, &refresh_token_hash).await? {
        return Err(ApiError::Unauthorized);
    }

    // Ensure the authenticated subject still maps to a real user account.
    if AuthRepo::find_user_by_id(&state.pool, user.user_id)
        .await?
        .is_none()
    {
        return Err(ApiError::Unauthorized);
    }

    // Hash new password
    let hashed_password = hash_password(&body.password)?;

    // Update password
    AuthRepo::update_user_password(&mut tx, user.user_id, &hashed_password).await?;

    // Revoke all existing refresh tokens
    AuthRepo::revoke_all_user_refresh_tokens(&mut tx, user.user_id).await?;

    tx.commit().await?;

    // Issue new tokens
    let access_token = create_access_token(
        user.user_id,
        &user.email,
        &state.env.jwt_secret,
        state.env.jwt_access_token_expiry_seconds,
    )?;

    let (refresh_token, jti) = create_refresh_token(
        user.user_id,
        &state.env.jwt_secret,
        state.env.jwt_refresh_token_expiry_seconds,
    )?;

    // Store new refresh token
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(jti.as_bytes());
        hex::encode(hasher.finalize())
    };
    let expires_at =
        Utc::now() + Duration::seconds(state.env.jwt_refresh_token_expiry_seconds as i64);

    AuthRepo::create_refresh_token(&state.pool, user.user_id, &token_hash, expires_at).await?;

    // Create cookies
    let access_cookie = create_access_token_cookie(
        &access_token,
        &state.env.cookie_domain,
        state.env.cookie_secure,
        state.env.jwt_access_token_expiry_seconds,
    );
    let refresh_cookie = create_refresh_token_cookie(
        &refresh_token,
        &state.env.cookie_domain,
        state.env.cookie_secure,
        state.env.jwt_refresh_token_expiry_seconds,
    );

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(SetPasswordResponse {
            message: "Password updated successfully.".to_string(),
        }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use actix_web::{App, http::StatusCode, test, web};
    use async_trait::async_trait;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use crate::auth::jwt::create_access_token;
    use crate::core::app_state::AppState;
    use crate::core::config::configure_routes;
    use crate::core::env::Env;
    use crate::core::error::ApiError;
    use crate::services::email::EmailSender;

    #[derive(Debug)]
    struct NoopEmailSender;

    #[async_trait]
    impl EmailSender for NoopEmailSender {
        async fn send_confirmation_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }

        async fn send_password_reset_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }
    }

    fn test_state() -> AppState {
        let pool = PgPoolOptions::new()
            .connect_lazy("postgres://postgres:postgres@localhost/gig_log_test")
            .expect("lazy test pool should be created");
        let env = Env {
            database_url: "postgres://postgres:postgres@localhost/gig_log_test".to_string(),
            cors_allowed_origin: "http://localhost:3000".to_string(),
            port: 0,
            jwt_secret: "test-jwt-secret".to_string(),
            jwt_access_token_expiry_seconds: 900,
            jwt_refresh_token_expiry_seconds: 604_800,
            resend_api_key: "test-resend-key".to_string(),
            resend_from_email: "test@giglog.dev".to_string(),
            auth_code_expiry_seconds: 600,
            cookie_domain: "localhost".to_string(),
            cookie_secure: false,
            log_level: "info".to_string(),
            log_http_body_enabled: true,
            log_http_max_body_bytes: 16_384,
        };

        AppState::with_email_sender(pool, env, Arc::new(NoopEmailSender))
    }

    #[actix_web::test]
    // Verifies signup rejects invalid payloads before handler DB logic executes.
    async fn sign_up_returns_bad_request_for_invalid_payload() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .configure(configure_routes),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/auth/sign-up")
            .set_json(json!({
                "first_name": "",
                "last_name": "",
                "email": "invalid-email",
                "password": "short",
                "confirm": ""
            }))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    // Verifies current-user endpoint returns unauthorized when no access cookie is present.
    async fn current_user_returns_unauthorized_without_cookie() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .configure(configure_routes),
        )
        .await;

        let request = test::TestRequest::get().uri("/auth/me").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies set-password requires a refresh-session cookie even with a valid access token.
    async fn set_password_returns_unauthorized_without_refresh_cookie() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "user@example.com",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("test access token should be created");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/auth/set-password")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .set_json(json!({
                "password": "new-password-123",
                "confirm": "new-password-123"
            }))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies set-password rejects unauthenticated requests before mutating credentials.
    async fn set_password_returns_unauthorized_without_cookie() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .configure(configure_routes),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/auth/set-password")
            .set_json(json!({
                "password": "new-password-123",
                "confirm": "new-password-123"
            }))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies forgot-password validation rejects malformed emails with a bad request.
    async fn forgot_password_returns_bad_request_for_invalid_email() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .configure(configure_routes),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/auth/forgot-password")
            .set_json(json!({ "email": "not-an-email" }))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    // Verifies logout succeeds without a refresh cookie and still clears auth cookies.
    async fn log_out_returns_ok_and_clears_cookies_without_refresh_cookie() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .configure(configure_routes),
        )
        .await;

        let request = test::TestRequest::post().uri("/auth/log-out").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let cookie_names: Vec<String> = response
            .response()
            .cookies()
            .map(|cookie| cookie.name().to_string())
            .collect();
        assert!(cookie_names.iter().any(|name| name == "access_token"));
        assert!(cookie_names.iter().any(|name| name == "refresh_token"));
    }
}
