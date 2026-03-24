use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn CompanyCreatePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Create Company"</h1>
        </MainLayout>
    }
}
