//! Job management route page components.

/// Provides the job-create page component.
pub mod create;
/// Provides the job-detail page component.
pub mod detail;
/// Provides the job-edit page component.
pub mod edit;
/// Provides the job-list page component.
pub mod list;

pub use create::JobCreatePage;
pub use detail::JobDetailPage;
pub use edit::JobEditPage;
pub use list::JobListPage;
