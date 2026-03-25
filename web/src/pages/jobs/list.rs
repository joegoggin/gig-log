use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn JobListPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Jobs"</h1>
        </MainLayout>
    }
}
