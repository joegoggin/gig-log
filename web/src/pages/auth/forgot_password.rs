use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Forgot Password"</h1>
        </AuthLayout>
    }
}
