use crate::handlers::auth::{SetPasswordRequest, SignUpRequest};

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

pub fn validate_signup_passwords_match(
    req: &SignUpRequest,
) -> Result<(), validator::ValidationError> {
    validate_passwords_match(&req.password, &req.confirm)
}

pub fn validate_set_password_match(
    req: &SetPasswordRequest,
) -> Result<(), validator::ValidationError> {
    validate_passwords_match(&req.password, &req.confirm)
}
