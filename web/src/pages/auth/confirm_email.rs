//! Page component for `ConfirmEmailPage`.

use gig_log_common::models::{error::ValidationError, user::ConfirmEmailRequest};
use leptos::{ev::SubmitEvent, prelude::*, reactive::spawn_local};
use leptos_router::hooks::use_navigate;

use crate::{
    api_client::ClientError,
    components::{
        Card, Form,
        button::{Button, ButtonType},
        text_input::TextInput,
    },
    contexts::{use_auth, use_notifications},
    layouts::auth::AuthLayout,
};

/// Renders the `ConfirmEmailPage` component.
///
/// # Returns
///
/// A Leptos view for the `ConfirmEmailPage` UI.
#[component]
pub fn ConfirmEmailPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();

    // State
    let is_loading = RwSignal::new(false);
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![]);
    let code = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        if is_loading.get_untracked() {
            return;
        }

        let request = ConfirmEmailRequest { code: code.get() };

        is_loading.set(true);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();
        let errors = errors;
        let is_loading = is_loading;

        spawn_local(async move {
            let next_errors = match auth.auth_requests().confirm_email(&request).await {
                Ok(response) => {
                    notifications.show_success("Email confirmed", response.message);
                    navigate("/auth/log-in", Default::default());
                    return;
                }
                Err(ClientError::Api(api_error)) => {
                    notifications.show_error("Email confirmation failed", api_error.message);
                    api_error.errors.unwrap_or_default()
                }
                Err(ClientError::Network(message)) => {
                    notifications.show_error(
                        "Email confirmation failed",
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
                title="Confirm Email"
                subtitle="Enter the code from your inbox to finish setting up your account."
            >
                <Form on_submit=handle_submit is_loading=is_loading>
                    <TextInput
                        name="code"
                        placeholder="Enter confirmation code"
                        errors=errors
                        value=code
                    />
                    <Button button_type=ButtonType::Submit>"Confirm Email"</Button>
                </Form>
            </Card>
        </AuthLayout>
    }
}
