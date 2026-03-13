use gig_log_common::models::{
    generic::MessageResponse,
    user::{
        ChangePasswordRequest, ConfirmEmailRequest, ForgotPasswordRequest, LogInRequest,
        RequestEmailChangeRequest, SetPasswordRequest, SignUpRequest, User,
        VerifyForgotPasswordRequest,
    },
};

use crate::api_client::{client::ApiClient, error::ClientError};

#[derive(Clone, Debug)]
pub struct AuthRequestRunner {
    client: ApiClient,
}

impl AuthRequestRunner {
    pub fn new() -> Self {
        Self {
            client: ApiClient::new(),
        }
    }

    pub async fn sign_up(&self, request: &SignUpRequest) -> Result<User, ClientError> {
        self.client.post("/auth/sign-up", Some(request)).await
    }

    pub async fn confirm_email(
        &self,
        request: &ConfirmEmailRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client.post("/auth/confirm-email", Some(request)).await
    }

    pub async fn log_in(&self, request: &LogInRequest) -> Result<User, ClientError> {
        self.client.post("/auth/log-in", Some(request)).await
    }

    pub async fn log_out(&self) -> Result<MessageResponse, ClientError> {
        self.client
            .post::<(), MessageResponse>("/auth/log-out", None)
            .await
    }

    pub async fn refresh(&self) -> Result<MessageResponse, ClientError> {
        self.client
            .post::<(), MessageResponse>("/auth/refresh", None)
            .await
    }

    pub async fn get_me(&self) -> Result<User, ClientError> {
        self.client.get("/auth/me").await
    }

    pub async fn forgot_password(
        &self,
        request: &ForgotPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/forgot-password", Some(request))
            .await
    }

    pub async fn verify_forgot_password(
        &self,
        request: &VerifyForgotPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/verify-forgot-password", Some(request))
            .await
    }

    pub async fn set_password(
        &self,
        request: &SetPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client.post("/auth/set-password", Some(request)).await
    }

    pub async fn request_change_password(&self) -> Result<MessageResponse, ClientError> {
        self.client
            .post::<(), MessageResponse>("/auth/request-change-password", None)
            .await
    }

    pub async fn change_password(
        &self,
        request: &ChangePasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/change-password", Some(request))
            .await
    }

    pub async fn request_email_change(
        &self,
        request: &RequestEmailChangeRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/request-email-change", Some(request))
            .await
    }

    pub async fn confirm_email_change(
        &self,
        request: &ConfirmEmailRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.client
            .post("/auth/confirm-email-change", Some(request))
            .await
    }
}
