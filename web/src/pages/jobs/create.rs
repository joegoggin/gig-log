use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn JobCreatePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Create Job"</h1>
        </MainLayout>
    }
}
