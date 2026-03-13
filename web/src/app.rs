use gig_log_frontend::{contexts::provide_auth_context, pages::*};
use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[component]
pub fn App() -> impl IntoView {
    provide_auth_context();

    view! {
        <Router>
            <main>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    // Auth routes
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/signup") view=SignupPage />
                    <Route path=path!("/forgot-password") view=ForgotPasswordPage />
                    <Route path=path!("/verify-forgot-password") view=VerifyForgotPasswordPage />
                    <Route path=path!("/set-password") view=SetPasswordPage />
                    <Route path=path!("/confirm-email") view=ConfirmEmailPage />

                    // Dashboard
                    <Route path=path!("/") view=DashboardPage />

                    // Companies
                    <Route path=path!("/companies") view=CompanyListPage />
                    <Route path=path!("/companies/new") view=CompanyCreatePage />
                    <Route path=path!("/companies/:id") view=CompanyDetailPage />
                    <Route path=path!("/companies/:id/edit") view=CompanyEditPage />

                    // Jobs
                    <Route path=path!("/jobs") view=JobListPage />
                    <Route path=path!("/jobs/new") view=JobCreatePage />
                    <Route path=path!("/jobs/:id") view=JobDetailPage />
                    <Route path=path!("/jobs/:id/edit") view=JobEditPage />

                    // Payments
                    <Route path=path!("/payments") view=PaymentListPage />
                    <Route path=path!("/payments/new") view=PaymentCreatePage />
                    <Route path=path!("/payments/:id") view=PaymentDetailPage />
                    <Route path=path!("/payments/:id/edit") view=PaymentEditPage />

                    // Settings
                    <Route path=path!("/settings") view=SettingsPage />
                </Routes>
            </main>
        </Router>
    }
}
