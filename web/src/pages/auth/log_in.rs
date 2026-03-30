//! Page component for `LoginPage`.

use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

/// Renders the `LoginPage` component.
///
/// # Returns
///
/// A Leptos view for the `LoginPage` UI.
#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Login"</h1>
        </AuthLayout>
    }
}
