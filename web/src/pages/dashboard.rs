use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Dashboard"</h1>
        </MainLayout>
    }
}
