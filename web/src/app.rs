use gig_log_frontend::{
    components::private_route::PrivateRoute,
    contexts::{provide_auth_context, provide_mobile_context, provide_notification_context},
    layouts::root::RootLayout,
    pages::*,
};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();
    provide_mobile_context();
    provide_notification_context();

    view! {
        <Router>
            <RootLayout>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    // Home
                    <Route path=path!("/") view=HomePage />

                    // Auth routes
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/signup") view=SignupPage />
                    <Route path=path!("/forgot-password") view=ForgotPasswordPage />
                    <Route path=path!("/verify-forgot-password") view=VerifyForgotPasswordPage />
                    <Route path=path!("/set-password") view=SetPasswordPage />
                    <Route path=path!("/confirm-email") view=ConfirmEmailPage />

                    // Dashboard
                    <PrivateRoute path=path!("/dashboard") view=DashboardPage />

                    // Companies
                    <PrivateRoute path=path!("/companies") view=CompanyListPage />
                    <PrivateRoute path=path!("/companies/new") view=CompanyCreatePage />
                    <PrivateRoute path=path!("/companies/:id") view=CompanyDetailPage />
                    <PrivateRoute path=path!("/companies/:id/edit") view=CompanyEditPage />

                    // Jobs
                    <PrivateRoute path=path!("/jobs") view=JobListPage />
                    <PrivateRoute path=path!("/jobs/new") view=JobCreatePage />
                    <PrivateRoute path=path!("/jobs/:id") view=JobDetailPage />
                    <PrivateRoute path=path!("/jobs/:id/edit") view=JobEditPage />

                    // Payments
                    <PrivateRoute path=path!("/payments") view=PaymentListPage />
                    <PrivateRoute path=path!("/payments/new") view=PaymentCreatePage />
                    <PrivateRoute path=path!("/payments/:id") view=PaymentDetailPage />
                    <PrivateRoute path=path!("/payments/:id/edit") view=PaymentEditPage />

                    // Settings
                    <PrivateRoute path=path!("/settings") view=SettingsPage />
                </Routes>
            </RootLayout>
        </Router>
    }
}
