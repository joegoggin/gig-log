use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn CompanyListPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Companies"</h1>
        </MainLayout>
    }
}
