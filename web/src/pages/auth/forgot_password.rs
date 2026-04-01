//! Page component for `ForgotPasswordPage`.

use gig_log_common::models::user::ForgotPasswordRequest;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::hooks::use_navigate;

use super::shared::{AuthFormCard, submit_auth_form, use_auth_form};
use crate::{
    components::{
        button::{Button, ButtonType},
        text_input::TextInput,
    },
    contexts::{use_auth, use_notifications},
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
    let form = use_auth_form();
    let email = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        let request = ForgotPasswordRequest { email: email.get() };

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();

        submit_auth_form(
            form,
            notifications,
            "Forgot password failed",
            async move { auth.forgot_password(&request).await },
            move |response| {
                notifications.show_success("Check your email", response.message);
                navigate("/auth/verify-forgot-password", Default::default());
            },
        );
    };

    view! {
        <AuthFormCard
            title="Forgot Password"
            subtitle="Enter your email and we will send a code to get you back in."
            is_loading=form.is_loading
            on_submit=handle_submit
        >
            <TextInput name="email" placeholder="Email" value=email errors=form.errors />
            <Button button_type=ButtonType::Submit>"Reset Password"</Button>
        </AuthFormCard>
    }
}
