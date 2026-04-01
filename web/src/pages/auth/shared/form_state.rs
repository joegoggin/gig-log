//! Shared auth-form submission state and request lifecycle helpers.

use std::future::Future;

use gig_log_common::models::error::ValidationError;
use leptos::{prelude::*, reactive::spawn_local};

use crate::{api_client::ClientError, contexts::NotificationContext};

/// Stores shared loading and validation state for auth forms.
#[derive(Debug, Clone, Copy)]
pub struct AuthFormState {
    /// Stores whether the form is currently submitting.
    pub is_loading: RwSignal<bool>,
    /// Stores validation errors returned by the backend.
    pub errors: RwSignal<Vec<ValidationError>>,
}

impl AuthFormState {
    /// Creates a new [`AuthFormState`] with default values.
    ///
    /// # Returns
    ///
    /// An initialized [`AuthFormState`].
    pub fn new() -> Self {
        Self {
            is_loading: RwSignal::new(false),
            errors: RwSignal::new(Vec::new()),
        }
    }

    /// Starts a submit cycle when the form is idle.
    ///
    /// # Returns
    ///
    /// A [`bool`] indicating whether submission should continue.
    pub fn start_submit(&self) -> bool {
        if self.is_loading.get_untracked() {
            return false;
        }

        self.is_loading.set(true);
        true
    }

    /// Marks submission as successful and clears validation errors.
    pub fn finish_success(&self) {
        self.is_loading.set(false);
        self.errors.set(Vec::new());
    }

    /// Marks submission as failed and stores validation errors.
    ///
    /// # Arguments
    ///
    /// * `errors` — Validation errors returned by the backend.
    pub fn finish_with_errors(&self, errors: Vec<ValidationError>) {
        self.is_loading.set(false);
        self.errors.set(errors);
    }
}

/// Creates a new auth form state container.
///
/// # Returns
///
/// An initialized [`AuthFormState`].
pub fn use_auth_form() -> AuthFormState {
    AuthFormState::new()
}

/// Submits an auth request and applies standardized result handling.
///
/// # Arguments
///
/// * `form_state` — Shared auth form state for loading and errors.
/// * `notifications` — Notification context used for user feedback.
/// * `failure_title` — Error toast title for failed requests.
/// * `request` — Request future that performs the API call.
/// * `on_success` — Callback run when the request succeeds.
pub fn submit_auth_form<Response, RequestFuture, OnSuccess>(
    form_state: AuthFormState,
    notifications: NotificationContext,
    failure_title: impl Into<String>,
    request: RequestFuture,
    on_success: OnSuccess,
) where
    RequestFuture: Future<Output = Result<Response, ClientError>> + 'static,
    OnSuccess: FnOnce(Response) + 'static,
{
    if !form_state.start_submit() {
        return;
    }

    let failure_title = failure_title.into();

    spawn_local(async move {
        match request.await {
            Ok(response) => {
                form_state.finish_success();
                on_success(response);
            }
            Err(ClientError::Api(api_error)) => {
                notifications.show_error(failure_title.clone(), api_error.message);
                form_state.finish_with_errors(api_error.errors.unwrap_or_default());
            }
            Err(ClientError::Network(message)) => {
                notifications.show_error(failure_title, format!("Network error: {}", message));
                form_state.finish_with_errors(Vec::new());
            }
        }
    });
}
