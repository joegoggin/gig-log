//! Page component for `LoginPage`.

use gig_log_common::models::user::LogInRequest;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::hooks::use_navigate;

use super::shared::{AuthFormCard, submit_auth_form, use_auth_form};
use crate::{
    components::{
        button::{Button, ButtonType},
        password_input::PasswordInput,
        text_input::TextInput,
    },
    contexts::{use_auth, use_notifications},
};

/// Renders the `LoginPage` component.
///
/// # Returns
///
/// A Leptos view for the `LoginPage` UI.
#[component]
pub fn LogInPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();

    // State
    let form = use_auth_form();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        let request = LogInRequest {
            email: email.get(),
            password: password.get(),
        };

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();

        submit_auth_form(
            form,
            notifications,
            "Log in failed",
            async move { auth.login(&request).await },
            move |_| {
                notifications.show_success("Logged in", "Welcome back.");
                navigate("/dashboard", Default::default());
            },
        );
    };

    view! {
        <AuthFormCard
            title="Log In"
            subtitle="Pick up where you left off and keep your freelance workflow moving."
            is_loading=form.is_loading
            on_submit=handle_submit
        >
            <TextInput name="email" placeholder="Email" errors=form.errors value=email />
            <PasswordInput
                name="password"
                placeholder="Password"
                errors=form.errors
                value=password
            />
            <Button button_type=ButtonType::Submit>Log In</Button>
        </AuthFormCard>
    }
}
