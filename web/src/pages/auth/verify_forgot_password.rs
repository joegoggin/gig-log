//! Page component for `VerifyForgotPasswordPage`.

use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

/// Renders the `VerifyForgotPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `VerifyForgotPasswordPage` UI.
#[component]
pub fn VerifyForgotPasswordPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Verify Forgot Password"</h1>
        </AuthLayout>
    }
}
