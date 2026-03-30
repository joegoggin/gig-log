//! Page component for `CompanyCreatePage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `CompanyCreatePage` component.
///
/// # Returns
///
/// A Leptos view for the `CompanyCreatePage` UI.
#[component]
pub fn CompanyCreatePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Create Company"</h1>
        </MainLayout>
    }
}
