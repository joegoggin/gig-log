use leptos::prelude::*;

use crate::layouts::main::MainLayout;

#[component]
pub fn PaymentDetailPage() -> impl IntoView {
    view! {
        <MainLayout>
            <h1>"Payment Detail"</h1>
        </MainLayout>
    }
}
