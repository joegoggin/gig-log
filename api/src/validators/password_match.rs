//! Custom validation functions for request payloads.
//!
//! This module provides validation helpers used with the `validator` crate
//! to perform cross-field validation that cannot be expressed with simple
//! field-level attributes.

use crate::routes::auth::{SetPasswordRequest, SignUpRequest};

/// Validates that two password fields match.
///
/// This is a private helper function used by the public validation functions
/// for specific request types.
///
/// # Arguments
///
/// * `password` - The password field value
/// * `confirm` - The confirmation password field value
///
/// # Errors
///
/// Returns a `ValidationError` with code `password_mismatch` if the passwords
/// do not match.
fn validate_passwords_match(
    password: &str,
    confirm: &str,
) -> Result<(), validator::ValidationError> {
    if password != confirm {
        let mut error = validator::ValidationError::new("password_mismatch");
        error.message = Some("Passwords do not match".into());
        return Err(error);
    }
    Ok(())
}

/// Validates that the password and confirm fields match in a sign-up request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`SignUpRequest`].
///
/// See [`sign_up`](crate::routes::auth::handlers::sign_up) for the handler
/// that uses this validation.
pub fn validate_signup_passwords_match(
    req: &SignUpRequest,
) -> Result<(), validator::ValidationError> {
    validate_passwords_match(&req.password, &req.confirm)
}

/// Validates that the password and confirm fields match in a set-password request.
///
/// Used with the `#[validate(custom(...))]` attribute on [`SetPasswordRequest`].
///
/// See [`set_password`](crate::routes::auth::handlers::set_password) for the handler
/// that uses this validation.
pub fn validate_set_password_match(
    req: &SetPasswordRequest,
) -> Result<(), validator::ValidationError> {
    validate_passwords_match(&req.password, &req.confirm)
}
