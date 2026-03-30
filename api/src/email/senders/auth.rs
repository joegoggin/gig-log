//! Authentication-related email senders.
//!
//! This module provides [`AuthSender`], which composes and delivers
//! emails for account actions such as email verification, password
//! resets, and credential change confirmations.

use crate::{core::error::ApiResult, email::client::EmailClient};

/// Sends authentication-related emails to users.
///
/// Composes and delivers emails for account actions such as email
/// verification, password resets, and credential change confirmations.
/// Each method formats a message containing a one-time code and delegates
/// delivery to the underlying [`EmailClient`].
pub struct AuthSender {
    /// Email client used to deliver messages.
    client: EmailClient,
    /// Recipient email address.
    to: String,
    /// One-time code included in the email body.
    code: String,
}

impl AuthSender {
    /// Creates a new [`AuthSender`].
    ///
    /// # Arguments
    ///
    /// * `client` — [`EmailClient`] used to send emails.
    /// * `to` — Recipient email address.
    /// * `code` — One-time code to include in the email body.
    ///
    /// # Returns
    ///
    /// A configured [`AuthSender`] instance.
    pub fn new(client: EmailClient, to: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            client,
            to: to.into(),
            code: code.into(),
        }
    }

    /// Sends an email verification code to the recipient.
    ///
    /// # Returns
    ///
    /// An empty [`ApiResult`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`](crate::core::error::ApiErrorResponse::InternalServerError) if email delivery fails.
    pub async fn send_email_verification(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Verify your email",
                &format!("Your verification code is: {}", self.code),
            )
            .await
    }

    /// Sends a password reset code to the recipient.
    ///
    /// # Returns
    ///
    /// An empty [`ApiResult`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`](crate::core::error::ApiErrorResponse::InternalServerError) if email delivery fails.
    pub async fn send_reset_password(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Reset your password",
                &format!("Your password reset code is: {}", self.code),
            )
            .await
    }

    /// Sends an email change confirmation code to the recipient.
    ///
    /// # Returns
    ///
    /// An empty [`ApiResult`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`](crate::core::error::ApiErrorResponse::InternalServerError) if email delivery fails.
    pub async fn send_email_change(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Confirm your email change",
                &format!("Your email change code is: {}", self.code),
            )
            .await
    }

    /// Sends a password change confirmation code to the recipient.
    ///
    /// # Returns
    ///
    /// An empty [`ApiResult`] on success.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`](crate::core::error::ApiErrorResponse::InternalServerError) if email delivery fails.
    pub async fn send_password_change(&self) -> ApiResult<()> {
        self.client
            .send_email(
                &self.to,
                "Confirm your password change",
                &format!("Your password change code is: {}", self.code),
            )
            .await
    }
}
