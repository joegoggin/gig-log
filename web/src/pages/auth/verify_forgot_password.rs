//! Page component for `VerifyForgotPasswordPage`.

use gig_log_common::models::{error::ValidationError, user::VerifyForgotPasswordRequest};
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

/// Renders the `VerifyForgotPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `VerifyForgotPasswordPage` UI.
#[component]
pub fn VerifyForgotPasswordPage() -> impl IntoView {
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

        let request = VerifyForgotPasswordRequest { code: code.get() };

        is_loading.set(true);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();
        let errors = errors;
        let is_loading = is_loading;

        spawn_local(async move {
            let next_errors = match auth.auth_requests().verify_forgot_password(&request).await {
                Ok(response) => {
                    notifications.show_success("Code verified", response.message);
                    navigate(
                        &format!("/auth/set-password?code={}", request.code),
                        Default::default(),
                    );
                    return;
                }
                Err(ClientError::Api(api_error)) => {
                    notifications.show_error("Code verification failed", api_error.message);
                    api_error.errors.unwrap_or_default()
                }
                Err(ClientError::Network(message)) => {
                    notifications.show_error(
                        "Code verification failed",
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
                title="Verify Reset Code"
                subtitle="Enter the code we sent to your email to continue."
            >
                <Form on_submit=handle_submit is_loading=is_loading>
                    <TextInput
                        name="code"
                        placeholder="Enter reset code"
                        value=code
                        errors=errors
                    />
                    <Button button_type=ButtonType::Submit>"Verify Code"</Button>
                </Form>
            </Card>
        </AuthLayout>
    }
}
