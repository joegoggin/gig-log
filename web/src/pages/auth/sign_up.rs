//! Page component for `SignupPage`.

use gig_log_common::models::user::SignUpRequest;
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
    let form = use_auth_form();
    let first_name = RwSignal::new(String::new());
    let last_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        let request = SignUpRequest {
            first_name: first_name.get(),
            last_name: last_name.get(),
            email: email.get(),
            password: password.get(),
            confirm_password: confirm_password.get(),
        };

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();

        submit_auth_form(
            form,
            notifications,
            "Sign up failed",
            async move { auth.signup(&request).await },
            move |response| {
                notifications.show_success("Account created", response.message);
                navigate("/auth/confirm-email", Default::default());
            },
        );
    };

    view! {
        <AuthFormCard
            title="Sign Up"
            subtitle="Start tracking companies, gigs, and payouts from one place."
            is_loading=form.is_loading
            on_submit=handle_submit
        >
            <TextInput
                name="first_name"
                placeholder="First Name"
                errors=form.errors
                value=first_name
            />
            <TextInput
                name="last_name"
                placeholder="Last Name"
                errors=form.errors
                value=last_name
            />
            <TextInput name="email" placeholder="Email" errors=form.errors value=email />
            <PasswordInput
                name="password"
                placeholder="Password"
                errors=form.errors
                value=password
            />
            <PasswordInput
                name="confirm_password"
                placeholder="Confirm Password"
                errors=form.errors
                value=confirm_password
            />
            <Button button_type=ButtonType::Submit>"Sign Up"</Button>
        </AuthFormCard>
    }
}
