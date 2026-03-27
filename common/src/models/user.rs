use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "validation")]
use crate::validators::user::{
    validate_change_password_match, validate_set_password_match, validate_signup_passwords_match,
};

/// A registered user account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,
    /// User's first name.
    pub first_name: String,
    /// User's last name.
    pub last_name: String,
    /// User's email address.
    pub email: String,
    /// Whether the user has confirmed their email address.
    pub email_confirmed: bool,
    /// Timestamp when the user account was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user account was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Request payload for creating a new user account.
///
/// When the `"validation"` feature is enabled, fields are validated and
/// `password` must match `confirm_password`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
#[cfg_attr(
    feature = "validation",
    validate(schema(function = "validate_signup_passwords_match"))
)]
pub struct SignUpRequest {
    /// User's first name.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "First name is required"))
    )]
    pub first_name: String,
    /// User's last name.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Last name is required"))
    )]
    pub last_name: String,
    /// User's email address.
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub email: String,
    /// Desired password (minimum 8 characters).
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 8, message = "Password must have at least 8 characters"))
    )]
    pub password: String,
    /// Must match `password`.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Confirm password is required"))
    )]
    pub confirm_password: String,
}

/// Request payload for logging in.
///
/// When the `"validation"` feature is enabled, fields are validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct LogInRequest {
    /// User's email address.
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub email: String,
    /// User's password.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Password is required"))
    )]
    pub password: String,
}

/// Request payload for changing the current user's password.
///
/// When the `"validation"` feature is enabled, fields are validated and
/// `new_password` must match `confirm_new_password`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
#[cfg_attr(
    feature = "validation",
    validate(schema(function = "validate_change_password_match"))
)]
pub struct ChangePasswordRequest {
    /// The user's current password.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Current password is required"))
    )]
    pub current_password: String,
    /// The desired new password.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "New password is required"))
    )]
    pub new_password: String,
    /// Must match `new_password`.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Confirm new password is required"))
    )]
    pub confirm_new_password: String,
    /// Verification code sent to the user.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Code is required"))
    )]
    pub code: String,
}

/// Request payload for initiating an email address change.
///
/// When the `"validation"` feature is enabled, `new_email` is validated as a valid email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct RequestEmailChangeRequest {
    /// The new email address to change to.
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub new_email: String,
}

/// Request payload for initiating a forgot-password flow.
///
/// When the `"validation"` feature is enabled, `email` is validated as a valid email.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct ForgotPasswordRequest {
    /// The email address associated with the account.
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub email: String,
}

/// Request payload for setting a new password after a forgot-password flow.
///
/// When the `"validation"` feature is enabled, fields are validated and
/// `new_password` must match `confirm_new_password`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
#[cfg_attr(
    feature = "validation",
    validate(schema(function = "validate_set_password_match"))
)]
pub struct SetPasswordRequest {
    /// Verification code sent to the user.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Code is required"))
    )]
    pub code: String,
    /// The desired new password.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "New password is required"))
    )]
    pub new_password: String,
    /// Must match `new_password`.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Confirm new password is required"))
    )]
    pub confirm_new_password: String,
}

/// Request payload for verifying a forgot-password code.
///
/// When the `"validation"` feature is enabled, `code` is validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct VerifyForgotPasswordRequest {
    /// Verification code sent to the user.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Code is required"))
    )]
    pub code: String,
}

/// Request payload for confirming a user's email address.
///
/// When the `"validation"` feature is enabled, `code` is validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct ConfirmEmailRequest {
    /// Authentication code sent to the user's email.
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Auth code is required"))
    )]
    pub code: String,
}
