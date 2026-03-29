//! Page component for `SetPasswordPage`.

use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

/// Renders the `SetPasswordPage` component.
///
/// # Returns
///
/// A Leptos view for the `SetPasswordPage` UI.
#[component]
pub fn SetPasswordPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Set Password"</h1>
        </AuthLayout>
    }
}
