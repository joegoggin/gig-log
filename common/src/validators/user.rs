//! Validators for user authentication request payloads.
//!
//! Each function checks that password and confirmation fields match,
//! returning a `validator::ValidationError` with code `"password_mismatch"`
//! on failure.

#[cfg(feature = "validation")]
use crate::models::user::{ChangePasswordRequest, SetPasswordRequest, SignUpRequest};

/// Validates that `password` and `confirm_password` match on a [`SignUpRequest`].
///
/// # Arguments
///
/// * `req` — The sign-up request to validate.
///
/// # Returns
///
/// `Ok(())` if the passwords match.
///
/// # Errors
///
/// Returns a [`ValidationError`](validator::ValidationError) with code
/// `"password_mismatch"` if the passwords do not match.
#[cfg(feature = "validation")]
pub fn validate_signup_passwords_match(
    req: &SignUpRequest,
) -> Result<(), validator::ValidationError> {
    if req.password != req.confirm_password {
        let mut error = validator::ValidationError::new("password_mismatch");
        error.message = Some("Passwords do not match".into());
        return Err(error);
    }

    Ok(())
}

/// Validates that `new_password` and `confirm_new_password` match on a [`SetPasswordRequest`].
///
/// # Arguments
///
/// * `req` — The set-password request to validate.
///
/// # Returns
///
/// `Ok(())` if the passwords match.
///
/// # Errors
///
/// Returns a [`ValidationError`](validator::ValidationError) with code
/// `"password_mismatch"` if the passwords do not match.
#[cfg(feature = "validation")]
pub fn validate_set_password_match(
    req: &SetPasswordRequest,
) -> Result<(), validator::ValidationError> {
    if req.new_password != req.confirm_new_password {
        let mut error = validator::ValidationError::new("password_mismatch");
        error.message = Some("Passwords do not match".into());
        return Err(error);
    }

    Ok(())
}

/// Validates that `new_password` and `confirm_new_password` match on a [`ChangePasswordRequest`].
///
/// # Arguments
///
/// * `req` — The change-password request to validate.
///
/// # Returns
///
/// `Ok(())` if the passwords match.
///
/// # Errors
///
/// Returns a [`ValidationError`](validator::ValidationError) with code
/// `"password_mismatch"` if the passwords do not match.
#[cfg(feature = "validation")]
pub fn validate_change_password_match(
    req: &ChangePasswordRequest,
) -> Result<(), validator::ValidationError> {
    if req.new_password != req.confirm_new_password {
        let mut error = validator::ValidationError::new("password_mismatch");
        error.message = Some("Passwords do not match".into());
        return Err(error);
    }

    Ok(())
}
