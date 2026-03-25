use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn SettingsPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Settings"</h1>
        </MainLayout>
    }
}
