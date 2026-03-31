//! Page component for `SignupPage`.

use gig_log_common::models::{error::ValidationError, user::SignUpRequest};
use leptos::{ev::SubmitEvent, prelude::*, reactive::spawn_local};
use leptos_router::hooks::use_navigate;

use crate::{
    api_client::ClientError,
    components::{
        Card, Form,
        button::{Button, ButtonType},
        password_input::PasswordInput,
        text_input::TextInput,
    },
    contexts::{use_auth, use_notifications},
    layouts::auth::AuthLayout,
    utils::class_name::ClassNameUtil,
};

/// Renders the `SignupPage` component.
///
/// # Returns
///
/// A Leptos view for the `SignupPage` UI.
#[component]
pub fn SignupPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();

    // State
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![]);
    let is_submitting = RwSignal::new(false);
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Classes
    let class_name = ClassNameUtil::new("sign-up-page", None);
    let sign_up_page = class_name.get_root_class();
    let card = class_name.get_sub_class("card");

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        if is_submitting.get_untracked() {
            return;
        }

        let request = SignUpRequest {
            first_name: first_name.get(),
            last_name: last_name.get(),
            email: email.get(),
            password: password.get(),
            confirm_password: confirm_password.get(),
        };

        is_submitting.set(true);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();
        let errors = errors;
        let is_submitting = is_submitting;

        spawn_local(async move {
            let next_errors = match auth.signup(&request).await {
                Ok(response) => {
                    notifications.show_success("Account created", response.message);
                    navigate("/auth/confirm-email", Default::default());
                    return;
                }
                Err(ClientError::Api(api_error)) => {
                    notifications.show_error("Sign up failed", api_error.message);
                    api_error.errors.unwrap_or_default()
                }
                Err(ClientError::Network(message)) => {
                    notifications
                        .show_error("Sign up failed", format!("Network error: {}", message));
                    vec![]
                }
            };

            is_submitting.set(false);
            errors.set(next_errors);
        });
    };

    view! {
        <AuthLayout class=sign_up_page>
            <Card class=card>
                <h1>"Sign Up"</h1>
                <p>"Start tracking companies, gigs, and payouts from one place."</p>
                <Form on_submit=handle_submit is_loading=is_submitting>
                    <TextInput
                        name="first_name"
                        placeholder="First Name"
                        errors=errors
                        value=first_name
                    />
                    <TextInput
                        name="last_name"
                        placeholder="Last Name"
                        errors=errors
                        value=last_name
                    />
                    <TextInput name="email" placeholder="Email" errors=errors value=email />
                    <PasswordInput
                        name="password"
                        placeholder="Password"
                        errors=errors
                        value=password
                    />
                    <PasswordInput
                        name="confirm_password"
                        placeholder="Confirm Password"
                        errors=errors
                        value=confirm_password
                    />
                    <Button button_type=ButtonType::Submit>"Sign Up"</Button>
                </Form>
            </Card>
        </AuthLayout>
    }
}
