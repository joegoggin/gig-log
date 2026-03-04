use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "validation")]
use crate::validators::user::{
    validate_change_password_match, validate_set_password_match, validate_signup_passwords_match,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub email_confirmed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
#[cfg_attr(
    feature = "validation",
    validate(schema(function = "validate_signup_passwords_match"))
)]
pub struct SignUpRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "First name is required"))
    )]
    pub first_name: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Last name is required"))
    )]
    pub last_name: String,
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub email: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 8, message = "Password must have at least 8 characters"))
    )]
    pub password: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Confirm password is required"))
    )]
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct LogInRequest {
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub email: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Password is required"))
    )]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
#[cfg_attr(
    feature = "validation",
    validate(schema(function = "validate_change_password_match"))
)]
pub struct ChangePasswordRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Current password is required"))
    )]
    pub current_password: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "New password is required"))
    )]
    pub new_password: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Confirm new password is required"))
    )]
    pub confirm_new_password: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Code is required"))
    )]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct RequestEmailChangeRequest {
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub new_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct ForgotPasswordRequest {
    #[cfg_attr(feature = "validation", validate(email(message = "Email is invalid")))]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
#[cfg_attr(
    feature = "validation",
    validate(schema(function = "validate_set_password_match"))
)]
pub struct SetPasswordRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Code is required"))
    )]
    pub code: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "New password is required"))
    )]
    pub new_password: String,
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Confirm new password is required"))
    )]
    pub confirm_new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct VerifyForgotPasswordRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Code is required"))
    )]
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "validation", derive(validator::Validate))]
pub struct ConfirmEmailRequest {
    #[cfg_attr(
        feature = "validation",
        validate(length(min = 1, message = "Auth code is required"))
    )]
    pub code: String,
}
