use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn SetPasswordPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Set Password"</h1>
        </AuthLayout>
    }
}
