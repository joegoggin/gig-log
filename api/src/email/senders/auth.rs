use crate::{core::error::ApiResult, email::client::EmailClient};

pub struct AuthSender {
    client: EmailClient,
    to: String,
    code: String,
}

impl AuthSender {
    pub async fn send_email_verification(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Verify your email",
                &format!("Your verification code is: {}", self.code),
            )
            .await
    }

    pub async fn send_reset_password(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Reset your password",
                &format!("Your password reset code is: {}", self.code),
            )
            .await
    }

    pub async fn send_email_change(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Confirm your email change",
                &format!("Your email change code is: {}", self.code),
            )
            .await
    }
}
