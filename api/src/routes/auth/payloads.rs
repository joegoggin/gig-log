//! Request and response payloads for authentication endpoints.
//!
//! This module contains all the data structures used for serializing and
//! deserializing HTTP request bodies and response payloads in the auth handlers.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::repository::auth::CurrentUser;
use crate::validators::password_match::{
    validate_set_password_match, validate_signup_passwords_match,
};

/// Request body for user registration.
///
/// Validates that passwords match and meet minimum length requirements.
///
/// See [`sign_up`](super::handlers::sign_up) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_signup_passwords_match"))]
pub struct SignUpRequest {
    /// User's first name.
    #[validate(length(min = 1, message = "First name is required"))]
    pub first_name: String,

    /// User's last name.
    #[validate(length(min = 1, message = "Last name is required"))]
    pub last_name: String,

    /// User's email address (must be unique).
    #[validate(email(message = "Email is invalid"))]
    pub email: String,

    /// User's chosen password (minimum 8 characters).
    #[validate(length(min = 8, message = "Password must have at least 8 characters"))]
    pub password: String,

    /// Password confirmation (must match `password`).
    #[validate(length(min = 1, message = "Confirm password is required"))]
    pub confirm: String,
}

/// Response body for successful user registration.
///
/// See [`sign_up`](super::handlers::sign_up) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct SignUpResponse {
    /// Success message instructing user to check email.
    pub message: String,
    /// The newly created user's unique identifier.
    pub user_id: Uuid,
}

/// Request body for email confirmation.
///
/// See [`confirm_email`](super::handlers::confirm_email) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmEmailRequest {
    /// The email address to confirm.
    #[validate(email(message = "Email is invalid"))]
    pub email: String,

    /// The confirmation code sent to the user's email.
    #[validate(length(min = 1, message = "Auth code is required"))]
    pub auth_code: String,
}

/// Response body for email confirmation.
///
/// See [`confirm_email`](super::handlers::confirm_email) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct ConfirmEmailResponse {
    /// Confirmation status message.
    pub message: String,
}

/// Request body for user login.
///
/// See [`log_in`](super::handlers::log_in) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct LogInRequest {
    /// User's email address.
    #[validate(email(message = "Email is invalid"))]
    pub email: String,

    /// User's password.
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,

    /// Whether the login session should persist across browser restarts.
    #[serde(default)]
    pub remember_me: bool,
}

/// Response body for successful login.
///
/// See [`log_in`](super::handlers::log_in) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct LogInResponse {
    /// Success message.
    pub message: String,
    /// The authenticated user's unique identifier.
    pub user_id: Uuid,
}

/// Response body for logout.
///
/// See [`log_out`](super::handlers::log_out) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct LogOutResponse {
    /// Success message.
    pub message: String,
}

/// Response body for successful token refresh.
///
/// See [`refresh_session`](super::handlers::refresh_session) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct RefreshSessionResponse {
    /// Success message.
    pub message: String,
}

/// Response body containing the authenticated user's information.
///
/// See [`current_user`](super::handlers::current_user) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    /// The current user's profile data.
    pub user: CurrentUser,
}

/// Request body for initiating password reset.
///
/// See [`forgot_password`](super::handlers::forgot_password) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    /// Email address of the account to reset.
    #[validate(email(message = "Email is invalid"))]
    pub email: String,
}

/// Response body for forgot password request.
///
/// See [`forgot_password`](super::handlers::forgot_password) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct ForgotPasswordResponse {
    /// Generic message (same whether email exists or not for security).
    pub message: String,
}

/// Request body for verifying a password reset code.
///
/// See [`verify_forgot_password`](super::handlers::verify_forgot_password) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyForgotPasswordRequest {
    /// Email address of the account.
    #[validate(email(message = "Email is invalid"))]
    pub email: String,

    /// The password reset code sent to the user's email.
    #[validate(length(min = 1, message = "Auth code is required"))]
    pub auth_code: String,
}

/// Response body for successful password reset code verification.
///
/// See [`verify_forgot_password`](super::handlers::verify_forgot_password) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct VerifyForgotPasswordResponse {
    /// Success message.
    pub message: String,
}

/// Request body for setting a new password.
///
/// Validates that passwords match and meet minimum length requirements.
///
/// See [`set_password`](super::handlers::set_password) for the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
#[validate(schema(function = "validate_set_password_match"))]
pub struct SetPasswordRequest {
    /// The new password (minimum 8 characters).
    #[validate(length(min = 8, message = "Password must have at least 8 characters"))]
    pub password: String,

    /// Password confirmation (must match `password`).
    #[validate(length(min = 1, message = "Confirm password is required"))]
    pub confirm: String,
}

/// Response body for successful password update.
///
/// See [`set_password`](super::handlers::set_password) for the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct SetPasswordResponse {
    /// Success message.
    pub message: String,
}
