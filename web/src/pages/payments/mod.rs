//! Payment management route page components.

/// Provides the payment-create page component.
pub mod create;
/// Provides the payment-detail page component.
pub mod detail;
/// Provides the payment-edit page component.
pub mod edit;
/// Provides the payment-list page component.
pub mod list;

pub use create::PaymentCreatePage;
pub use detail::PaymentDetailPage;
pub use edit::PaymentEditPage;
pub use list::PaymentListPage;
