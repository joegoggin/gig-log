use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn SignupPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Sign Up"</h1>
        </AuthLayout>
    }
}
