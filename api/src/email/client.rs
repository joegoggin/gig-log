//! Core email client for the Resend API.
//!
//! This module provides [`EmailClient`], a thin wrapper around
//! [`reqwest::Client`] that authenticates with the Resend API and
//! delivers plain-text email messages.

use reqwest::Client;
use serde_json::json;

use crate::core::config::Config;
use crate::core::error::{ApiErrorResponse, ApiResult};

/// Default Resend API base URL used by the production email client.
const DEFAULT_RESEND_BASE_URL: &str = "https://api.resend.com";

/// HTTP client for sending emails through the Resend API.
///
/// Wraps a [`reqwest::Client`] with Resend API credentials and provides
/// a single [`send_email`](Self::send_email) method for delivering messages.
#[derive(Debug, Clone)]
pub struct EmailClient {
    /// Underlying HTTP client used for API requests.
    client: Client,
    /// Resend API key for authentication.
    api_key: String,
    /// Sender email address included in outgoing messages.
    from_email: String,
    /// Base URL for the email delivery API.
    base_url: String,
}

impl EmailClient {
    /// Creates a new [`EmailClient`] from application configuration.
    ///
    /// # Arguments
    ///
    /// * `config` — Application [`Config`] providing the Resend API key
    ///   and sender email address.
    ///
    /// # Returns
    ///
    /// A configured [`EmailClient`] ready to send emails.
    pub fn new(config: &Config) -> Self {
        Self::new_with_base_url(config, DEFAULT_RESEND_BASE_URL)
    }

    /// Creates a new [`EmailClient`] with a custom API base URL.
    ///
    /// This is primarily used by integration tests to point requests at a
    /// wiremock server instead of the live Resend API.
    ///
    /// # Arguments
    ///
    /// * `config` — Application [`Config`] providing the Resend API key
    ///   and sender email address.
    /// * `base_url` — Base URL for the email API endpoint.
    ///
    /// # Returns
    ///
    /// A configured [`EmailClient`] ready to send emails.
    pub fn new_with_base_url(config: &Config, base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: config.resend_api_key.clone(),
            from_email: config.resend_from_email.clone(),
            base_url: base_url.into(),
        }
    }

    /// Sends a plain-text email to a single recipient via the Resend API.
    ///
    /// # Arguments
    ///
    /// * `to` — Recipient email address.
    /// * `subject` — Email subject line.
    /// * `body` — Plain-text email body.
    ///
    /// # Returns
    ///
    /// An empty [`ApiResult`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`] if the HTTP request
    /// to the Resend API fails.
    pub async fn send_email(&self, to: &str, subject: &str, body: &str) -> ApiResult<()> {
        let endpoint = format!("{}/emails", self.base_url.trim_end_matches('/'));

        self.client
            .post(endpoint)
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
