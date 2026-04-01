//! Page component for `ForgotPasswordPage`.

use gig_log_common::models::{error::ValidationError, user::ForgotPasswordRequest};
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

/// Renders the `ForgotPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `ForgotPasswordPage` UI.
#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();

    // State
    let is_loading = RwSignal::new(false);
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![]);
    let email = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        if is_loading.get_untracked() {
            return;
        }

        let request = ForgotPasswordRequest { email: email.get() };

        is_loading.set(true);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();
        let errors = errors;
        let is_loading = is_loading;

        spawn_local(async move {
            let next_errors = match auth.auth_requests().forgot_password(&request).await {
                Ok(response) => {
                    notifications.show_success("Check your email", response.message);
                    navigate("/auth/verify-forgot-password", Default::default());
                    return;
                }
                Err(ClientError::Api(api_error)) => {
                    notifications.show_error("Forgot password failed", api_error.message);
                    api_error.errors.unwrap_or_default()
                }
                Err(ClientError::Network(message)) => {
                    notifications.show_error(
                        "Forgot password failed",
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
                title="Forgot Password"
                subtitle="Enter your email and we will send a code to get you back in."
            >
                <Form on_submit=handle_submit is_loading=is_loading>
                    <TextInput name="email" placeholder="Email" value=email errors=errors />
                    <Button button_type=ButtonType::Submit>"Reset Password"</Button>
                </Form>
            </Card>
        </AuthLayout>
    }
}
