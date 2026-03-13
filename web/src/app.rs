use gig_log_frontend::{
    components::{core::notifications::Notifications, protected_route::ProtectedRoute},
    contexts::{provide_auth_context, provide_notification_context},
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
    provide_notification_context();

    view! {
        <Router>
            <main>
                <Notifications />
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    // Auth routes
                    <Route path=path!("/login") view=LoginPage />
                    <Route path=path!("/signup") view=SignupPage />
                    <Route path=path!("/forgot-password") view=ForgotPasswordPage />
                    <Route path=path!("/verify-forgot-password") view=VerifyForgotPasswordPage />
                    <Route path=path!("/set-password") view=SetPasswordPage />
                    <Route path=path!("/confirm-email") view=ConfirmEmailPage />

                    // Dashboard
                    <Route
                        path=path!("/")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <DashboardPage />
                                </ProtectedRoute>
                            }
                        }
                    />

                    // Companies
                    <Route
                        path=path!("/companies")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <CompanyListPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/companies/new")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <CompanyCreatePage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/companies/:id")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <CompanyDetailPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/companies/:id/edit")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <CompanyEditPage />
                                </ProtectedRoute>
                            }
                        }
                    />

                    // Jobs
                    <Route
                        path=path!("/jobs")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <JobListPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/jobs/new")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <JobCreatePage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/jobs/:id")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <JobDetailPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/jobs/:id/edit")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <JobEditPage />
                                </ProtectedRoute>
                            }
                        }
                    />

                    // Payments
                    <Route
                        path=path!("/payments")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <PaymentListPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/payments/new")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <PaymentCreatePage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/payments/:id")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <PaymentDetailPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                    <Route
                        path=path!("/payments/:id/edit")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <PaymentEditPage />
                                </ProtectedRoute>
                            }
                        }
                    />

                    // Settings
                    <Route
                        path=path!("/settings")
                        view=|| {
                            view! {
                                <ProtectedRoute>
                                    <SettingsPage />
                                </ProtectedRoute>
                            }
                        }
                    />
                </Routes>
            </main>
        </Router>
    }
}
