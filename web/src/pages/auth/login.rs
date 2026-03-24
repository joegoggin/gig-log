use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Login"</h1>
        </AuthLayout>
    }
}
