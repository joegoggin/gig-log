//! Page component for `JobDetailPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `JobDetailPage` component.
///
/// # Returns
///
/// A Leptos view for the `JobDetailPage` UI.
#[component]
pub fn JobDetailPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Job Detail"</h1>
        </MainLayout>
    }
}
