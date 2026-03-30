//! Page component for `CompanyListPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `CompanyListPage` component.
///
/// # Returns
///
/// A Leptos view for the `CompanyListPage` UI.
#[component]
pub fn CompanyListPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Companies"</h1>
        </MainLayout>
    }
}
