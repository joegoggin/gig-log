//! Page component for `PaymentEditPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `PaymentEditPage` component.
///
/// # Returns
///
/// A Leptos view for the `PaymentEditPage` UI.
#[component]
pub fn PaymentEditPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Edit Payment"</h1>
        </MainLayout>
    }
}
