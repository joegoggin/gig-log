//! Page component for `CompanyDetailPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `CompanyDetailPage` component.
///
/// # Returns
///
/// A Leptos view for the `CompanyDetailPage` UI.
#[component]
pub fn CompanyDetailPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Company Detail"</h1>
        </MainLayout>
    }
}
