//! Page component for `SettingsPage`.

use leptos::prelude::*;

use crate::layouts::main::MainLayout;

/// Renders the `SettingsPage` component.
///
/// # Returns
///
/// A Leptos view for the `SettingsPage` UI.
#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Settings"</h1>
        </MainLayout>
    }
}
