//! Authentication-related route page components.

/// Provides shared helpers for auth page implementations.
mod shared;

/// Provides the confirm-email page component.
pub mod confirm_email;
/// Provides the forgot-password page component.
pub mod forgot_password;
/// Provides the login page component.
pub mod log_in;
/// Provides the set-password page component.
pub mod set_password;
/// Provides the sign-up page component.
pub mod sign_up;
/// Provides the forgot-password verification page component.
pub mod verify_forgot_password;

pub use confirm_email::*;
pub use forgot_password::*;
pub use log_in::*;
pub use set_password::*;
pub use sign_up::*;
pub use verify_forgot_password::*;
