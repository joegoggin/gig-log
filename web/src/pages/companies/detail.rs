use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn CompanyDetailPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Company Detail"</h1>
        </MainLayout>
    }
}
