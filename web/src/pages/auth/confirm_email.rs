//! Page component for `ConfirmEmailPage`.

use gig_log_common::models::user::ConfirmEmailRequest;
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
    let form = use_auth_form();
    let code = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        let request = ConfirmEmailRequest { code: code.get() };

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();

        submit_auth_form(
            form,
            notifications,
            "Email confirmation failed",
            async move { auth.confirm_email(&request).await },
            move |response| {
                notifications.show_success("Email confirmed", response.message);
                navigate("/auth/log-in", Default::default());
            },
        );
    };

    view! {
        <AuthFormCard
            title="Confirm Email"
            subtitle="Enter the code from your inbox to finish setting up your account."
            is_loading=form.is_loading
            on_submit=handle_submit
        >
            <TextInput
                name="code"
                placeholder="Enter confirmation code"
                errors=form.errors
                value=code
            />
            <Button button_type=ButtonType::Submit>"Confirm Email"</Button>
        </AuthFormCard>
    }
}
