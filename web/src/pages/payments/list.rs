use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn PaymentListPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Payments"</h1>
        </MainLayout>
    }
}
