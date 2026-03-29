//! Page component for `DashboardPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `DashboardPage` component.
///
/// # Returns
///
/// A Leptos view for the `DashboardPage` UI.
#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Dashboard"</h1>
        </MainLayout>
    }
}
