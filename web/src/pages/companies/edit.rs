use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn CompanyEditPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Edit Company"</h1>
        </MainLayout>
    }
}
