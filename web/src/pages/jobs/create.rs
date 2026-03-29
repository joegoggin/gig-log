//! Page component for `JobCreatePage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `JobCreatePage` component.
///
/// # Returns
///
/// A Leptos view for the `JobCreatePage` UI.
#[component]
pub fn JobCreatePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Create Job"</h1>
        </MainLayout>
    }
}
