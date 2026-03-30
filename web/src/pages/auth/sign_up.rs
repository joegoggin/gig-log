//! Page component for `SignupPage`.

use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

/// Renders the `SignupPage` component.
///
/// # Returns
///
/// A Leptos view for the `SignupPage` UI.
#[component]
pub fn SignupPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Sign Up"</h1>
        </AuthLayout>
    }
}
