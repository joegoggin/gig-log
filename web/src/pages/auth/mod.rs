//! Authentication-related route page components.

/// Provides the confirm-email page component.
pub mod confirm_email;
/// Provides the forgot-password page component.
pub mod forgot_password;
/// Provides the login page component.
pub mod login;
/// Provides the set-password page component.
pub mod set_password;
/// Provides the sign-up page component.
pub mod signup;
/// Provides the forgot-password verification page component.
pub mod verify_forgot_password;

pub use confirm_email::*;
pub use forgot_password::*;
pub use login::*;
pub use set_password::*;
pub use signup::*;
pub use verify_forgot_password::*;
