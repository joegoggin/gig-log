//! Authentication context state and helper actions.

use gig_log_common::models::{
    generic::MessageResponse,
    user::{
        ConfirmEmailRequest, ForgotPasswordRequest, LogInRequest, SetPasswordRequest,
        SignUpRequest, User, VerifyForgotPasswordRequest,
    },
};
use leptos::{prelude::*, reactive::spawn_local};

use crate::api_client::{AuthRequestRunner, ClientError};

/// Stores authentication state and auth API helper methods.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// Stores the authenticated user when available.
    pub user: RwSignal<Option<User>>,
    /// Stores whether auth state is currently loading.
    pub loading: RwSignal<bool>,
    /// Stores auth request helpers used by this context.
    auth_requests: AuthRequestRunner,
}

impl AuthContext {
    /// Creates a new [`AuthContext`] with default state.
    ///
    /// # Returns
    ///
    /// An initialized [`AuthContext`] with no authenticated user.
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            loading: RwSignal::new(true),
            auth_requests: AuthRequestRunner::new(),
        }
    }

    /// Returns whether a user is currently authenticated.
    ///
    /// # Returns
    ///
    /// A [`bool`] indicating whether `user` is present.
    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

    /// Checks auth status by requesting the current user.
    ///
    /// Updates [`Self::user`] and [`Self::loading`] based on the request
    /// result.
    pub async fn check_auth(&self) {
        self.loading.set(true);

        match self.auth_requests.get_me().await {
            Ok(user) => {
                self.user.set(Some(user));
            }
            Err(_) => {
                self.user.set(None);
            }
        }

        self.loading.set(false);
    }

    /// Logs in a user and updates context state.
    ///
    /// # Arguments
    ///
    /// * `request` — Credentials payload for the login request.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing the authenticated [`User`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the login request fails.
    pub async fn login(&self, request: &LogInRequest) -> Result<User, ClientError> {
        let user = self.auth_requests.log_in(request).await?;
        self.user.set(Some(user.clone()));
        Ok(user)
    }

    /// Registers a new user account.
    ///
    /// # Arguments
    ///
    /// * `request` — Sign-up payload for account creation.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a backend [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the sign-up request fails.
    pub async fn signup(&self, request: &SignUpRequest) -> Result<MessageResponse, ClientError> {
        self.auth_requests.sign_up(request).await
    }

    /// Confirms a user's email address with a verification code.
    ///
    /// # Arguments
    ///
    /// * `request` — Email confirmation payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a backend [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the confirmation request fails.
    pub async fn confirm_email(
        &self,
        request: &ConfirmEmailRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.auth_requests.confirm_email(request).await
    }

    /// Requests a forgot-password verification code.
    ///
    /// # Arguments
    ///
    /// * `request` — Forgot-password payload containing the user email.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a backend [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn forgot_password(
        &self,
        request: &ForgotPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.auth_requests.forgot_password(request).await
    }

    /// Verifies a forgot-password code before setting a new password.
    ///
    /// # Arguments
    ///
    /// * `request` — Forgot-password verification payload.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a backend [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn verify_forgot_password(
        &self,
        request: &VerifyForgotPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.auth_requests.verify_forgot_password(request).await
    }

    /// Sets a new password using a verified forgot-password code.
    ///
    /// # Arguments
    ///
    /// * `request` — New-password payload and verification code.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing a backend [`MessageResponse`] on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the request fails.
    pub async fn set_password(
        &self,
        request: &SetPasswordRequest,
    ) -> Result<MessageResponse, ClientError> {
        self.auth_requests.set_password(request).await
    }

    /// Logs out the current user and clears auth state.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing `()` on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the logout request fails.
    pub async fn logout(&self) -> Result<(), ClientError> {
        self.auth_requests.log_out().await?;
        self.user.set(None);
        Ok(())
    }

    /// Refreshes the authenticated user session.
    ///
    /// # Returns
    ///
    /// A [`Result`] containing `()` on success.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] if the refresh request fails.
    pub async fn refresh(&self) -> Result<(), ClientError> {
        let user = self.auth_requests.refresh().await?;
        self.user.set(Some(user));
        Ok(())
    }
}

/// Provides and initializes the shared [`AuthContext`].
///
/// # Returns
///
/// The created [`AuthContext`] that was inserted into Leptos context.
pub fn provide_auth_context() -> AuthContext {
    let auth = AuthContext::new();
    provide_context(auth.clone());

    let auth_clone = auth.clone();
    spawn_local(async move {
        auth_clone.check_auth().await;
    });

    auth
}

/// Retrieves the shared authentication context.
///
/// # Returns
///
/// The current [`AuthContext`] from Leptos context.
pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>()
        .expect("AuthContext not provided. Wrap your app with provide_auth_context()")
}
