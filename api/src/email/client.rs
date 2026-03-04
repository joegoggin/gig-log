use reqwest::Client;
use serde_json::json;

use crate::core::config::Config;
use crate::core::error::{ApiErrorResponse, ApiResult};

#[derive(Debug, Clone)]
pub struct EmailClient {
    client: Client,
    api_key: String,
    from_email: String,
}

impl EmailClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            api_key: config.resend_api_key.clone(),
            from_email: config.resend_from_email.clone(),
        }
    }

    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> ApiResult<()> {
        self.client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "from": self.from_email,
                "to": [to],
                "subject": subject,
                "text": body,
            }))
            .send()
            .await
            .map_err(|error| ApiErrorResponse::InternalServerError(error.to_string()))?;

        Ok(())
    }
}
