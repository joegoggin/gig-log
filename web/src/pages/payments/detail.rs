//! Page component for `PaymentDetailPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `PaymentDetailPage` component.
///
/// # Returns
///
/// A Leptos view for the `PaymentDetailPage` UI.
#[component]
pub fn PaymentDetailPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Payment Detail"</h1>
        </MainLayout>
    }
}
