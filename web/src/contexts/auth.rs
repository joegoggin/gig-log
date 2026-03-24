use gig_log_common::models::{
    generic::MessageResponse,
    user::{LogInRequest, SignUpRequest, User},
};
use leptos::{prelude::*, reactive::spawn_local};

use crate::api_client::{AuthRequestRunner, ClientError};

#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user: RwSignal<Option<User>>,
    pub loading: RwSignal<bool>,
    auth_requests: AuthRequestRunner,
}

impl AuthContext {
    pub fn new() -> Self {
        Self {
            user: RwSignal::new(None),
            loading: RwSignal::new(true),
            auth_requests: AuthRequestRunner::new(),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.user.get().is_some()
    }

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

    pub async fn login(&self, request: &LogInRequest) -> Result<User, ClientError> {
        let user = self.auth_requests.log_in(request).await?;
        self.user.set(Some(user.clone()));
        Ok(user)
    }

    pub async fn signup(&self, request: &SignUpRequest) -> Result<MessageResponse, ClientError> {
        self.auth_requests.sign_up(request).await
    }

    pub async fn logout(&self) -> Result<(), ClientError> {
        self.auth_requests.log_out().await?;
        self.user.set(None);
        Ok(())
    }

    pub async fn refresh(&self) -> Result<(), ClientError> {
        let user = self.auth_requests.refresh().await?;
        self.user.set(Some(user));
        Ok(())
    }

    pub fn auth_requests(&self) -> &AuthRequestRunner {
        &self.auth_requests
    }
}

pub fn provide_auth_context() -> AuthContext {
    let auth = AuthContext::new();
    provide_context(auth.clone());

    let auth_clone = auth.clone();
    spawn_local(async move {
        auth_clone.check_auth().await;
    });

    auth
}

pub fn use_auth() -> AuthContext {
    use_context::<AuthContext>()
        .expect("AuthContext not provided. Wrap your app with provide_auth_context()")
}
