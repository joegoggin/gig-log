//! Page component for `VerifyForgotPasswordPage`.

use gig_log_common::models::user::VerifyForgotPasswordRequest;
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
    let form = use_auth_form();
    let code = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        let request = VerifyForgotPasswordRequest { code: code.get() };
        let next_path = format!("/auth/set-password?code={}", request.code);

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();

        submit_auth_form(
            form,
            notifications,
            "Code verification failed",
            async move { auth.verify_forgot_password(&request).await },
            move |response| {
                notifications.show_success("Code verified", response.message);
                navigate(&next_path, Default::default());
            },
        );
    };

    view! {
        <AuthFormCard
            title="Verify Reset Code"
            subtitle="Enter the code we sent to your email to continue."
            is_loading=form.is_loading
            on_submit=handle_submit
        >
            <TextInput
                name="code"
                placeholder="Enter reset code"
                value=code
                errors=form.errors
            />
            <Button button_type=ButtonType::Submit>"Verify Code"</Button>
        </AuthFormCard>
    }
}
