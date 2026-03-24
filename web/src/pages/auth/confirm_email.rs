use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn ConfirmEmailPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Confirm Email"</h1>
        </AuthLayout>
    }
}
