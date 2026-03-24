use leptos::prelude::*;

use crate::layouts::auth::AuthLayout;

#[component]
pub fn VerifyForgotPasswordPage() -> impl IntoView {
    view! {
        <AuthLayout>
            <h1>"Verify Forgot Password"</h1>
        </AuthLayout>
    }
}
