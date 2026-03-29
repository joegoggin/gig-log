//! Page component for `JobListPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `JobListPage` component.
///
/// # Returns
///
/// A Leptos view for the `JobListPage` UI.
#[component]
pub fn JobListPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Jobs"</h1>
        </MainLayout>
    }
}
