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

#[cfg(test)]
mod tests {
    use super::{validate_set_password_match, validate_signup_passwords_match};
    use crate::routes::auth::{SetPasswordRequest, SignUpRequest};

    #[test]
    // Verifies signup password matching validation accepts equal values.
    fn signup_password_validator_accepts_matching_passwords() {
        let request = SignUpRequest {
            first_name: "Jane".to_string(),
            last_name: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            password: "password123".to_string(),
            confirm: "password123".to_string(),
        };

        assert!(validate_signup_passwords_match(&request).is_ok());
    }

    #[test]
    // Verifies set-password validation returns a mismatch error when fields differ.
    fn set_password_validator_rejects_mismatch() {
        let request = SetPasswordRequest {
            password: "password123".to_string(),
            confirm: "different".to_string(),
        };

        let result = validate_set_password_match(&request);
        let error = result.expect_err("validator should reject mismatch");

        assert_eq!(error.code.as_ref(), "password_mismatch");
        assert_eq!(error.message.as_deref(), Some("Passwords do not match"));
    }
}
