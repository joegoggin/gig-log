//! Shared API error types and HTTP error response mapping.
//!
//! This module defines domain-level API errors and converts them to
//! standardized JSON error responses with appropriate status codes.

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use serde_json::json;
use std::fmt;

/// Standard result type returned by HTTP handlers.
pub type ApiResult<T> = Result<T, ApiError>;

/// Application error variants exposed by API handlers.
#[derive(Debug)]
pub enum ApiError {
    /// Email/password combination is invalid.
    InvalidCredentials,
    /// User attempted auth flow before confirming email.
    EmailNotConfirmed,
    /// Registration attempted with an email that already exists.
    EmailAlreadyExists,
    /// Provided auth/confirmation code is invalid.
    InvalidAuthCode,
    /// Provided auth/confirmation code has expired.
    AuthCodeExpired,
    /// JWT token is valid structurally but no longer valid due to expiry.
    TokenExpired,
    /// JWT token is malformed or otherwise invalid.
    TokenInvalid,
    /// Request requires authentication and no valid session/token was provided.
    Unauthorized,
    /// A requested resource was not found.
    NotFound(String),

    /// Request payload failed validation with a custom message.
    ValidationError(String),
    /// Password and password confirmation values did not match.
    PasswordMismatch,

    /// Database operation failed.
    DatabaseError(String),
    /// Upstream email provider operation failed.
    EmailServiceError(String),
    /// Unclassified internal application error.
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::InvalidCredentials => write!(f, "Invalid email or password"),
            ApiError::EmailNotConfirmed => write!(f, "Please confirm your email address"),
            ApiError::EmailAlreadyExists => write!(f, "An account with this email already exists"),
            ApiError::InvalidAuthCode => write!(f, "Invalid authentication code"),
            ApiError::AuthCodeExpired => write!(f, "Authentication code has expired"),
            ApiError::TokenExpired => write!(f, "Token has expired"),
            ApiError::TokenInvalid => write!(f, "Invalid token"),
            ApiError::Unauthorized => write!(f, "Unauthorized"),
            ApiError::NotFound(msg) => write!(f, "{}", msg),
            ApiError::ValidationError(msg) => write!(f, "{}", msg),
            ApiError::PasswordMismatch => write!(f, "Passwords do not match"),
            ApiError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ApiError::EmailServiceError(msg) => write!(f, "Email service error: {}", msg),
            ApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ApiError::EmailNotConfirmed => StatusCode::FORBIDDEN,
            ApiError::EmailAlreadyExists => StatusCode::CONFLICT,
            ApiError::InvalidAuthCode => StatusCode::BAD_REQUEST,
            ApiError::AuthCodeExpired => StatusCode::BAD_REQUEST,
            ApiError::TokenExpired => StatusCode::UNAUTHORIZED,
            ApiError::TokenInvalid => StatusCode::UNAUTHORIZED,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::PasswordMismatch => StatusCode::BAD_REQUEST,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::EmailServiceError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let error_code = match self {
            ApiError::InvalidCredentials => "INVALID_CREDENTIALS",
            ApiError::EmailNotConfirmed => "EMAIL_NOT_CONFIRMED",
            ApiError::EmailAlreadyExists => "EMAIL_ALREADY_EXISTS",
            ApiError::InvalidAuthCode => "INVALID_AUTH_CODE",
            ApiError::AuthCodeExpired => "AUTH_CODE_EXPIRED",
            ApiError::TokenExpired => "TOKEN_EXPIRED",
            ApiError::TokenInvalid => "TOKEN_INVALID",
            ApiError::Unauthorized => "UNAUTHORIZED",
            ApiError::NotFound(_) => "NOT_FOUND",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::PasswordMismatch => "PASSWORD_MISMATCH",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::EmailServiceError(_) => "EMAIL_SERVICE_ERROR",
            ApiError::InternalError(_) => "INTERNAL_ERROR",
        };

        HttpResponse::build(self.status_code()).json(json!({
            "error": {
                "code": error_code,
                "message": self.to_string()
            }
        }))
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for ApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => ApiError::TokenExpired,
            _ => ApiError::TokenInvalid,
        }
    }
}

impl From<argon2::password_hash::Error> for ApiError {
    fn from(_err: argon2::password_hash::Error) -> Self {
        ApiError::InvalidCredentials
    }
}
