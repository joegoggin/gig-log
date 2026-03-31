//! Page component for `LoginPage`.

use gig_log_common::models::error::ValidationError;
use gig_log_common::models::user::LogInRequest;
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

/// Renders the `LoginPage` component.
///
/// # Returns
///
/// A Leptos view for the `LoginPage` UI.
#[component]
pub fn LoginPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();

    // State
    let is_loading = RwSignal::new(false);
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![]);
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());

    // Classes
    let class_name = ClassNameUtil::new("login-page", None);
    let login_page = class_name.get_root_class();
    let card = class_name.get_sub_class("card");

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        if is_loading.get_untracked() {
            return;
        }

        let request = LogInRequest {
            email: email.get(),
            password: password.get(),
        };

        is_loading.set(true);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();
        let errors = errors;
        let is_loading = is_loading;

        spawn_local(async move {
            let next_errors = match auth.login(&request).await {
                Ok(_) => {
                    notifications.show_success("Logged in", "Welcome back.");
                    navigate("/dashboard", Default::default());
                    return;
                }
                Err(ClientError::Api(api_error)) => {
                    notifications.show_error("Log in failed", api_error.message);
                    api_error.errors.unwrap_or_default()
                }
                Err(ClientError::Network(message)) => {
                    notifications
                        .show_error("Log in failed", format!("Network error: {}", message));
                    vec![]
                }
            };

            is_loading.set(false);
            errors.set(next_errors);
        });
    };

    view! {
        <AuthLayout class=login_page>
            <Card class=card>
                <h1>"Login"</h1>
                <p>"Pick up where you left off and keep your freelance workflow moving."</p>
                <Form on_submit=handle_submit is_loading=is_loading>
                    <TextInput name="email" placeholder="Email" errors=errors value=email />
                    <PasswordInput
                        name="password"
                        placeholder="Password"
                        errors=errors
                        value=password
                    />
                    <Button button_type=ButtonType::Submit>Log In</Button>
                </Form>
            </Card>
        </AuthLayout>
    }
}
