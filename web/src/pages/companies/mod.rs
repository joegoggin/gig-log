//! Company management route page components.

/// Provides the company-create page component.
pub mod create;
/// Provides the company-detail page component.
pub mod detail;
/// Provides the company-edit page component.
pub mod edit;
/// Provides the company-list page component.
pub mod list;

pub use create::CompanyCreatePage;
pub use detail::CompanyDetailPage;
pub use edit::CompanyEditPage;
pub use list::CompanyListPage;
