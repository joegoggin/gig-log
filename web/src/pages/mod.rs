//! Route-level page components.

/// Provides authentication page components.
pub mod auth;
/// Provides company management pages.
pub mod companies;
/// Provides the dashboard page.
pub mod dashboard;
/// Provides the landing/home page.
pub mod home;
/// Provides job management pages.
pub mod jobs;
/// Provides the fallback 404 page.
pub mod not_found;
/// Provides payment management pages.
pub mod payments;
/// Provides the user settings page.
pub mod settings;

pub use auth::*;
pub use companies::{CompanyCreatePage, CompanyDetailPage, CompanyEditPage, CompanyListPage};
pub use dashboard::*;
pub use home::*;
pub use jobs::{JobCreatePage, JobDetailPage, JobEditPage, JobListPage};
pub use not_found::*;
pub use payments::{PaymentCreatePage, PaymentDetailPage, PaymentEditPage, PaymentListPage};
pub use settings::*;
