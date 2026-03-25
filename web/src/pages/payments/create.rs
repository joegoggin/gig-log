use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn PaymentCreatePage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Create Payment"</h1>
        </MainLayout>
    }
}
