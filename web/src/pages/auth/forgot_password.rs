//! Page component for `ForgotPasswordPage`.

use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

/// Renders the `ForgotPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `ForgotPasswordPage` UI.
#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Forgot Password"</h1>
        </AuthLayout>
    }
}
