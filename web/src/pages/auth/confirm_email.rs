//! Page component for `ConfirmEmailPage`.

use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

/// Renders the `ConfirmEmailPage` component.
///
/// # Returns
///
/// A Leptos view for the `ConfirmEmailPage` UI.
#[component]
pub fn ConfirmEmailPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Confirm Email"</h1>
        </AuthLayout>
    }
}
