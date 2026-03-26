use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn JobEditPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Edit Job"</h1>
        </MainLayout>
    }
}
