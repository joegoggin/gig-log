//! Page component for `PaymentListPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `PaymentListPage` component.
///
/// # Returns
///
/// A Leptos view for the `PaymentListPage` UI.
#[component]
pub fn PaymentListPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Payments"</h1>
        </MainLayout>
    }
}
