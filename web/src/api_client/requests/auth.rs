//! Authentication request helpers for frontend API calls.

use gig_log_common::models::{
    generic::MessageResponse,
    user::{
        ChangePasswordRequest, ConfirmEmailRequest, ForgotPasswordRequest, LogInRequest,
        RequestEmailChangeRequest, SetPasswordRequest, SignUpRequest, User,
        VerifyForgotPasswordRequest,
    },
};

use crate::api_client::{client::ApiClient, error::ClientError};

/// Executes authentication-related API requests.
#[derive(Clone, Debug)]
pub struct AuthRequestRunner {
    /// Stores the low-level API client used for requests.
    client: ApiClient,
}

impl AuthRequestRunner {
    /// Creates a new [`AuthRequestRunner`].
    ///
    /// # Returns
    ///
    /// An initialized [`AuthRequestRunner`].
    pub fn new() -> Self {
        Self {
            client: ApiClient::new(),
        }
    }

    /// Sends the sign-up request.
    ///
    /// # Arguments
    ///
    /// * `request` — Sign-up request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn sign_up(&self, request: &SignUpRequest) -> Result<MessageResponse, ClientError> {
        self.client.post("/auth/sign-up", Some(request)).await
    }

    /// Sends the confirm-email request.
    ///
    /// # Arguments
    ///
    /// * `request` — Email confirmation request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn confirm_email(
        &self,
        request: &ConfirmEmailRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client.post("/auth/confirm-email", Some(request)).await
    }

    /// Sends the login request.
    ///
    /// # Arguments
    ///
    /// * `request` — Login request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the authenticated [`User`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn log_in(&self, request: &LogInRequest) -> Result<User, ClientError> {
        self.client.post("/auth/log-in", Some(request)).await
    }

    /// Sends the logout request.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing `()` on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn log_out(&self) -> Result<(), ClientError> {
        self.client
            .post_no_content::<()>("/auth/log-out", None)
            .await
    }

    /// Sends the refresh-token request.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the refreshed [`User`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn refresh(&self) -> Result<User, ClientError> {
        self.client.post::<(), User>("/auth/refresh", None).await
    }

    /// Requests the current authenticated user.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the current [`User`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn get_me(&self) -> Result<User, ClientError> {
        self.client.get("/auth/me").await
    }

    /// Sends the forgot-password request.
    ///
    /// # Arguments
    ///
    /// * `request` — Forgot-password request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn forgot_password(
        &self,
        request: &ForgotPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/forgot-password", Some(request))
            .await
    }

    /// Sends the forgot-password verification request.
    ///
    /// # Arguments
    ///
    /// * `request` — Verification request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn verify_forgot_password(
        &self,
        request: &VerifyForgotPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/verify-forgot-password", Some(request))
            .await
    }

    /// Sends the set-password request.
    ///
    /// # Arguments
    ///
    /// * `request` — Set-password request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn set_password(
        &self,
        request: &SetPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client.post("/auth/set-password", Some(request)).await
    }

    /// Requests a password-change verification email.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn request_change_password(&self) -> Result<MessageResponse, ClientError> {
        self.client
            .post::<(), MessageResponse>("/auth/request-change-password", None)
            .await
    }

    /// Sends the change-password request.
    ///
    /// # Arguments
    ///
    /// * `request` — Change-password request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn change_password(
        &self,
        request: &ChangePasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/change-password", Some(request))
            .await
    }

    /// Sends the request-email-change request.
    ///
    /// # Arguments
    ///
    /// * `request` — Email-change request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn request_email_change(
        &self,
        request: &RequestEmailChangeRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/request-email-change", Some(request))
            .await
    }

    /// Sends the confirm-email-change request.
    ///
    /// # Arguments
    ///
    /// * `request` — Email confirmation request payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn confirm_email_change(
        &self,
        request: &ConfirmEmailRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/confirm-email-change", Some(request))
            .await
    }
}
