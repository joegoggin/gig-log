use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "validation")]
use crate::validators::user::{validate_set_password_match, validate_signup_passwords_match};

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
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
    pub confirm_new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEmailChangeRequest {
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

#[cfg(all(test, feature = "validation"))]
mod tests {
    use serde_json::json;
    use validator::Validate;

    use super::{ConfirmEmailRequest, SignUpRequest};

    #[test]
    fn sign_up_request_requires_matching_password_fields() {
        let req = SignUpRequest {
            first_name: "Joe".to_string(),
            last_name: "Goggin".to_string(),
            email: "joe@example.com".to_string(),
            password: "password123".to_string(),
            confirm_password: "different".to_string(),
        };

        let errors = req
            .validate()
            .expect_err("validation should fail for password mismatch");

        assert!(
            errors.errors().contains_key("__all__"),
            "schema validation error should be present"
        );
    }

    #[test]
    fn confirm_email_request_requires_code() {
        let req = ConfirmEmailRequest {
            code: "".to_string(),
        };

        let errors = req
            .validate()
            .expect_err("validation should fail for empty code");

        assert!(errors.errors().contains_key("code"));
    }

    #[test]
    fn sign_up_request_missing_first_name_fails_deserialization() {
        let raw = json!({
            "last_name": "Goggin",
            "email": "joe@example.com",
            "password": "Password1234$",
            "confirm_password": "Password1234$"
        });

        let err = serde_json::from_value::<SignUpRequest>(raw)
            .expect_err("request should fail deserialization without first_name");

        assert!(err.to_string().contains("missing field `first_name`"));
    }

    #[test]
    fn confirm_email_request_missing_code_fails_deserialization() {
        let err = serde_json::from_value::<ConfirmEmailRequest>(json!({}))
            .expect_err("request should fail deserialization without code");

        assert!(err.to_string().contains("missing field `code`"));
    }
}
