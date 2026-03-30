//! Page component for `JobEditPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `JobEditPage` component.
///
/// # Returns
///
/// A Leptos view for the `JobEditPage` UI.
#[component]
pub fn JobEditPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Edit Job"</h1>
        </MainLayout>
    }
}
