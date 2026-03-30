//! Page component for `PaymentCreatePage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `PaymentCreatePage` component.
///
/// # Returns
///
/// A Leptos view for the `PaymentCreatePage` UI.
#[component]
pub fn PaymentCreatePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Create Payment"</h1>
        </MainLayout>
    }
}
