//! Page component for `SetPasswordPage`.

use gig_log_common::models::{error::ValidationError, user::SetPasswordRequest};
use leptos::{ev::SubmitEvent, prelude::*, reactive::spawn_local};
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::{
    api_client::ClientError,
    components::{
        Card, Form,
        button::{Button, ButtonType},
        password_input::PasswordInput,
    },
    contexts::{use_auth, use_notifications},
    layouts::auth::AuthLayout,
};

/// Renders the `SetPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `SetPasswordPage` UI.
#[component]
pub fn SetPasswordPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();
    let query_map = use_query_map();

    // State
    let is_loading = RwSignal::new(false);
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![]);
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        if is_loading.get_untracked() {
            return;
        }

        let Some(code) = query_map.with(|params| params.get("code")) else {
            notifications.show_error(
                "Reset password failed",
                "Reset code is missing. Request a new reset code.",
            );
            return;
        };

        let request = SetPasswordRequest {
            code,
            new_password: password.get(),
            confirm_new_password: confirm_password.get(),
        };

        is_loading.set(true);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();
        let errors = errors;
        let is_loading = is_loading;

        spawn_local(async move {
            let next_errors = match auth.auth_requests().set_password(&request).await {
                Ok(response) => {
                    notifications.show_success("Password reset", response.message);
                    navigate("/auth/log-in", Default::default());
                    return;
                }
                Err(ClientError::Api(api_error)) => {
                    notifications.show_error("Reset password failed", api_error.message);
                    api_error.errors.unwrap_or_default()
                }
                Err(ClientError::Network(message)) => {
                    notifications.show_error(
                        "Reset password failed",
                        format!("Network error: {}", message),
                    );
                    vec![]
                }
            };

            is_loading.set(false);
            errors.set(next_errors);
        });
    };

    view! {
        <AuthLayout>
            <Card
                title="Set Password"
                subtitle="Choose a strong password so your account stays secure."
            >
                <Form on_submit=handle_submit is_loading=is_loading>
                    <PasswordInput
                        name="new_password"
                        placeholder="Password"
                        value=password
                        errors=errors
                    />
                    <PasswordInput
                        name="confirm_new_password"
                        placeholder="Confirm Password"
                        value=confirm_password
                        errors=errors
                    />
                    <Button button_type=ButtonType::Submit>"Set Password"</Button>
                </Form>
            </Card>
        </AuthLayout>
    }
}
