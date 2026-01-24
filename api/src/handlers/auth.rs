use actix_web::{HttpResponse, post, web};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::auth::codes::{generate_auth_code, hash_code, verify_code};
use crate::auth::cookies::{
    clear_access_token_cookie, clear_refresh_token_cookie, create_access_token_cookie,
    create_refresh_token_cookie,
};
use crate::auth::jwt::{create_access_token, create_refresh_token};
use crate::auth::middleware::AuthenticatedUser;
use crate::auth::password::{hash_password, verify_password};
use crate::core::env::Env;
use crate::core::error::{ApiError, ApiResult};
use crate::services::email::EmailService;

#[derive(Debug, Deserialize)]
pub struct SignUpRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
    pub confirm: String,
}

#[derive(Debug, Serialize)]
pub struct SignUpResponse {
    pub message: String,
    pub user_id: Uuid,
}

#[post("/auth/sign-up")]
pub async fn sign_up(
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    body: web::Json<SignUpRequest>,
) -> ApiResult<HttpResponse> {
    // Validate passwords match
    if body.password != body.confirm {
        return Err(ApiError::PasswordMismatch);
    }

    // Check if email already exists
    let existing_user = sqlx::query_scalar!(r#"SELECT id FROM users WHERE email = $1"#, body.email)
        .fetch_optional(pool.get_ref())
        .await?;

    if existing_user.is_some() {
        return Err(ApiError::EmailAlreadyExists);
    }

    // Hash password
    let hashed_password = hash_password(&body.password)?;

    // Create user
    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (first_name, last_name, email, hashed_password, email_confirmed)
        VALUES ($1, $2, $3, $4, false)
        RETURNING id
        "#,
        body.first_name,
        body.last_name,
        body.email,
        hashed_password
    )
    .fetch_one(pool.get_ref())
    .await?;

    // Generate and store auth code
    let code = generate_auth_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + Duration::seconds(env.auth_code_expiry_seconds as i64);

    sqlx::query!(
        r#"
        INSERT INTO auth_codes (user_id, code_hash, code_type, expires_at)
        VALUES ($1, $2, 'email_confirmation', $3)
        "#,
        user_id,
        code_hash,
        expires_at
    )
    .execute(pool.get_ref())
    .await?;

    // Send confirmation email
    let email_service = EmailService::new(&env.resend_api_key, &env.resend_from_email);
    email_service
        .send_confirmation_email(&body.email, &body.first_name, &code)
        .await?;

    Ok(HttpResponse::Created().json(SignUpResponse {
        message: "Account created. Please check your email for a confirmation code.".to_string(),
        user_id,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ConfirmEmailRequest {
    pub email: String,
    pub auth_code: String,
}

#[derive(Debug, Serialize)]
pub struct ConfirmEmailResponse {
    pub message: String,
}

#[post("/auth/confirm-email")]
pub async fn confirm_email(
    pool: web::Data<Pool<Postgres>>,
    body: web::Json<ConfirmEmailRequest>,
) -> ApiResult<HttpResponse> {
    // Find user by email
    let user = sqlx::query!(
        r#"SELECT id, email_confirmed FROM users WHERE email = $1"#,
        body.email
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::InvalidCredentials)?;

    if user.email_confirmed {
        return Ok(HttpResponse::Ok().json(ConfirmEmailResponse {
            message: "Email already confirmed.".to_string(),
        }));
    }

    // Find valid auth code
    let auth_code = sqlx::query!(
        r#"
        SELECT id, code_hash, expires_at
        FROM auth_codes
        WHERE user_id = $1
          AND code_type = 'email_confirmation'
          AND used = false
          AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user.id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::AuthCodeExpired)?;

    // Verify code
    if !verify_code(&body.auth_code, &auth_code.code_hash) {
        return Err(ApiError::InvalidAuthCode);
    }

    // Mark code as used and confirm email in a transaction
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"UPDATE auth_codes SET used = true WHERE id = $1"#,
        auth_code.id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        r#"UPDATE users SET email_confirmed = true, updated_at = NOW() WHERE id = $1"#,
        user.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ConfirmEmailResponse {
        message: "Email confirmed successfully.".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct LogInRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LogInResponse {
    pub message: String,
    pub user_id: Uuid,
}

#[post("/auth/log-in")]
pub async fn log_in(
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    body: web::Json<LogInRequest>,
) -> ApiResult<HttpResponse> {
    // Find user by email
    let user = sqlx::query!(
        r#"SELECT id, email, hashed_password, email_confirmed FROM users WHERE email = $1"#,
        body.email
    )
    .fetch_optional(pool.get_ref())
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
        &env.jwt_secret,
        env.jwt_access_token_expiry_seconds,
    )?;

    // Create refresh token
    let (refresh_token, jti) = create_refresh_token(
        user.id,
        &env.jwt_secret,
        env.jwt_refresh_token_expiry_seconds,
    )?;

    // Hash and store refresh token
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(jti.as_bytes());
        hex::encode(hasher.finalize())
    };
    let expires_at = Utc::now() + Duration::seconds(env.jwt_refresh_token_expiry_seconds as i64);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.id,
        token_hash,
        expires_at
    )
    .execute(pool.get_ref())
    .await?;

    // Create cookies
    let access_cookie =
        create_access_token_cookie(&access_token, &env.cookie_domain, env.cookie_secure);
    let refresh_cookie =
        create_refresh_token_cookie(&refresh_token, &env.cookie_domain, env.cookie_secure);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(LogInResponse {
            message: "Logged in successfully.".to_string(),
            user_id: user.id,
        }))
}

#[derive(Debug, Serialize)]
pub struct LogOutResponse {
    pub message: String,
}

#[post("/auth/log-out")]
pub async fn log_out(
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    req: actix_web::HttpRequest,
) -> ApiResult<HttpResponse> {
    // Try to revoke refresh token if present
    if let Some(refresh_cookie) = req.cookie("refresh_token") {
        if let Ok(claims) =
            crate::auth::jwt::decode_refresh_token(refresh_cookie.value(), &env.jwt_secret)
        {
            let token_hash = {
                let mut hasher = Sha256::new();
                hasher.update(claims.jti.as_bytes());
                hex::encode(hasher.finalize())
            };

            let _ = sqlx::query!(
                r#"UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1"#,
                token_hash
            )
            .execute(pool.get_ref())
            .await;
        }
    }

    // Clear cookies
    let clear_access = clear_access_token_cookie(&env.cookie_domain);
    let clear_refresh = clear_refresh_token_cookie(&env.cookie_domain);

    Ok(HttpResponse::Ok()
        .cookie(clear_access)
        .cookie(clear_refresh)
        .json(LogOutResponse {
            message: "Logged out successfully.".to_string(),
        }))
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    pub message: String,
}

#[post("/auth/forgot-password")]
pub async fn forgot_password(
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    body: web::Json<ForgotPasswordRequest>,
) -> ApiResult<HttpResponse> {
    // Always return success to prevent email enumeration
    let response = ForgotPasswordResponse {
        message: "If an account with this email exists, a password reset code has been sent."
            .to_string(),
    };

    // Find user by email
    let user = match sqlx::query!(
        r#"SELECT id, first_name FROM users WHERE email = $1"#,
        body.email
    )
    .fetch_optional(pool.get_ref())
    .await?
    {
        Some(user) => user,
        None => return Ok(HttpResponse::Ok().json(response)),
    };

    // Invalidate any existing password reset codes
    sqlx::query!(
        r#"
        UPDATE auth_codes
        SET used = true
        WHERE user_id = $1 AND code_type = 'password_reset' AND used = false
        "#,
        user.id
    )
    .execute(pool.get_ref())
    .await?;

    // Generate and store new auth code
    let code = generate_auth_code();
    let code_hash = hash_code(&code);
    let expires_at = Utc::now() + Duration::seconds(env.auth_code_expiry_seconds as i64);

    sqlx::query!(
        r#"
        INSERT INTO auth_codes (user_id, code_hash, code_type, expires_at)
        VALUES ($1, $2, 'password_reset', $3)
        "#,
        user.id,
        code_hash,
        expires_at
    )
    .execute(pool.get_ref())
    .await?;

    // Send password reset email
    let email_service = EmailService::new(&env.resend_api_key, &env.resend_from_email);
    let _ = email_service
        .send_password_reset_email(&body.email, &user.first_name, &code)
        .await;

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
pub struct VerifyForgotPasswordRequest {
    pub email: String,
    pub auth_code: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyForgotPasswordResponse {
    pub message: String,
}

#[post("/auth/verify-forgot-password")]
pub async fn verify_forgot_password(
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    body: web::Json<VerifyForgotPasswordRequest>,
) -> ApiResult<HttpResponse> {
    // Find user by email
    let user = sqlx::query!(
        r#"SELECT id, email FROM users WHERE email = $1"#,
        body.email
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::InvalidCredentials)?;

    // Find valid auth code
    let auth_code = sqlx::query!(
        r#"
        SELECT id, code_hash, expires_at
        FROM auth_codes
        WHERE user_id = $1
          AND code_type = 'password_reset'
          AND used = false
          AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
        user.id
    )
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or(ApiError::AuthCodeExpired)?;

    // Verify code
    if !verify_code(&body.auth_code, &auth_code.code_hash) {
        return Err(ApiError::InvalidAuthCode);
    }

    // Mark code as used
    sqlx::query!(
        r#"UPDATE auth_codes SET used = true WHERE id = $1"#,
        auth_code.id
    )
    .execute(pool.get_ref())
    .await?;

    // Issue tokens to allow password reset
    let access_token = create_access_token(
        user.id,
        &user.email,
        &env.jwt_secret,
        env.jwt_access_token_expiry_seconds,
    )?;

    let (refresh_token, jti) = create_refresh_token(
        user.id,
        &env.jwt_secret,
        env.jwt_refresh_token_expiry_seconds,
    )?;

    // Store refresh token
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(jti.as_bytes());
        hex::encode(hasher.finalize())
    };
    let expires_at = Utc::now() + Duration::seconds(env.jwt_refresh_token_expiry_seconds as i64);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.id,
        token_hash,
        expires_at
    )
    .execute(pool.get_ref())
    .await?;

    // Create cookies
    let access_cookie =
        create_access_token_cookie(&access_token, &env.cookie_domain, env.cookie_secure);
    let refresh_cookie =
        create_refresh_token_cookie(&refresh_token, &env.cookie_domain, env.cookie_secure);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(VerifyForgotPasswordResponse {
            message: "Code verified. You can now set a new password.".to_string(),
        }))
}

#[derive(Debug, Deserialize)]
pub struct SetPasswordRequest {
    pub password: String,
    pub confirm: String,
}

#[derive(Debug, Serialize)]
pub struct SetPasswordResponse {
    pub message: String,
}

#[post("/auth/set-password")]
pub async fn set_password(
    user: AuthenticatedUser,
    pool: web::Data<Pool<Postgres>>,
    env: web::Data<Env>,
    body: web::Json<SetPasswordRequest>,
) -> ApiResult<HttpResponse> {
    // Validate passwords match
    if body.password != body.confirm {
        return Err(ApiError::PasswordMismatch);
    }

    // Hash new password
    let hashed_password = hash_password(&body.password)?;

    // Start transaction
    let mut tx = pool.begin().await?;

    // Update password
    sqlx::query!(
        r#"UPDATE users SET hashed_password = $1, updated_at = NOW() WHERE id = $2"#,
        hashed_password,
        user.user_id
    )
    .execute(&mut *tx)
    .await?;

    // Revoke all existing refresh tokens
    sqlx::query!(
        r#"UPDATE refresh_tokens SET revoked = true WHERE user_id = $1"#,
        user.user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Issue new tokens
    let access_token = create_access_token(
        user.user_id,
        &user.email,
        &env.jwt_secret,
        env.jwt_access_token_expiry_seconds,
    )?;

    let (refresh_token, jti) = create_refresh_token(
        user.user_id,
        &env.jwt_secret,
        env.jwt_refresh_token_expiry_seconds,
    )?;

    // Store new refresh token
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(jti.as_bytes());
        hex::encode(hasher.finalize())
    };
    let expires_at = Utc::now() + Duration::seconds(env.jwt_refresh_token_expiry_seconds as i64);

    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
        user.user_id,
        token_hash,
        expires_at
    )
    .execute(pool.get_ref())
    .await?;

    // Create cookies
    let access_cookie =
        create_access_token_cookie(&access_token, &env.cookie_domain, env.cookie_secure);
    let refresh_cookie =
        create_refresh_token_cookie(&refresh_token, &env.cookie_domain, env.cookie_secure);

    Ok(HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(SetPasswordResponse {
            message: "Password updated successfully.".to_string(),
        }))
}
