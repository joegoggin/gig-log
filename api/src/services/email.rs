use resend_rs::{Resend, types::CreateEmailBaseOptions};

use crate::core::error::ApiError;

pub struct EmailService {
    client: Resend,
    from_email: String,
}

impl EmailService {
    pub fn new(api_key: &str, from_email: &str) -> Self {
        let client = Resend::new(api_key);
        Self {
            client,
            from_email: from_email.to_string(),
        }
    }

    pub async fn send_confirmation_email(
        &self,
        to_email: &str,
        first_name: &str,
        code: &str,
    ) -> Result<(), ApiError> {
        let html_body = format!(
            r#"
            <h2>Confirm your GigLog account</h2>
            <p>Hi {},</p>
            <p>Your confirmation code is: <strong>{}</strong></p>
            <p>This code expires in 10 minutes.</p>
            <p>If you didn't create an account, please ignore this email.</p>
            "#,
            first_name, code
        );

        let email = CreateEmailBaseOptions::new(
            &self.from_email,
            vec![to_email.to_string()],
            "Confirm your GigLog account",
        )
        .with_html(&html_body);

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| ApiError::EmailServiceError(e.to_string()))?;

        Ok(())
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        first_name: &str,
        code: &str,
    ) -> Result<(), ApiError> {
        let html_body = format!(
            r#"
            <h2>Reset your GigLog password</h2>
            <p>Hi {},</p>
            <p>Your password reset code is: <strong>{}</strong></p>
            <p>This code expires in 10 minutes.</p>
            <p>If you didn't request a password reset, please ignore this email.</p>
            "#,
            first_name, code
        );

        let email = CreateEmailBaseOptions::new(
            &self.from_email,
            vec![to_email.to_string()],
            "Reset your GigLog password",
        )
        .with_html(&html_body);

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| ApiError::EmailServiceError(e.to_string()))?;

        Ok(())
    }
}
