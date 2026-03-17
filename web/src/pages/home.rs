use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>Hello</h1>
        </MainLayout>
    }
}
