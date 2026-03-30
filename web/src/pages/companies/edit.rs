//! Page component for `CompanyEditPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `CompanyEditPage` component.
///
/// # Returns
///
/// A Leptos view for the `CompanyEditPage` UI.
#[component]
pub fn CompanyEditPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Edit Company"</h1>
        </MainLayout>
    }
}
