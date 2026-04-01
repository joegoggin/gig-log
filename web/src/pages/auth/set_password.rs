//! Page component for `SetPasswordPage`.

use gig_log_common::models::user::SetPasswordRequest;
use leptos::{ev::SubmitEvent, prelude::*};
use leptos_router::hooks::{use_navigate, use_query_map};

use super::shared::{AuthFormCard, submit_auth_form, use_auth_form};
use crate::{
    components::{
        button::{Button, ButtonType},
        password_input::PasswordInput,
    },
    contexts::{use_auth, use_notifications},
};

/// Renders the `SetPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `SetPasswordPage` UI.
#[component]
pub fn SetPasswordPage() -> impl IntoView {
    // Context
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();
    let query_map = use_query_map();

    // State
    let form = use_auth_form();
    let password = RwSignal::new(String::new());
    let confirm_password = RwSignal::new(String::new());

    // Event Handlers
    let handle_submit = move |_: SubmitEvent| {
        let Some(code) = query_map.with(|params| params.get("code")) else {
            notifications.show_error(
                "Reset password failed",
                "Reset code is missing. Request a new reset code.",
            );
            return;
        };

        let request = SetPasswordRequest {
            code,
            new_password: password.get(),
            confirm_new_password: confirm_password.get(),
        };

        let auth = auth.clone();
        let notifications = notifications;
        let navigate = navigate.clone();

        submit_auth_form(
            form,
            notifications,
            "Reset password failed",
            async move { auth.set_password(&request).await },
            move |response| {
                notifications.show_success("Password reset", response.message);
                navigate("/auth/log-in", Default::default());
            },
        );
    };

    view! {
        <AuthFormCard
            title="Set Password"
            subtitle="Choose a strong password so your account stays secure."
            is_loading=form.is_loading
            on_submit=handle_submit
        >
            <PasswordInput
                name="new_password"
                placeholder="Password"
                value=password
                errors=form.errors
            />
            <PasswordInput
                name="confirm_new_password"
                placeholder="Confirm Password"
                value=confirm_password
                errors=form.errors
            />
            <Button button_type=ButtonType::Submit>"Set Password"</Button>
        </AuthFormCard>
    }
}
