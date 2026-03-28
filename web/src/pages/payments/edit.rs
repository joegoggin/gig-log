use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn PaymentEditPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Edit Payment"</h1>
        </MainLayout>
    }
}
