#[cfg(feature = "validation")]
use crate::models::user::{ChangePasswordRequest, SetPasswordRequest, SignUpRequest};

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
