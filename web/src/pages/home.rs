use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>Hello</h1>
        </AuthLayout>
    }
}
