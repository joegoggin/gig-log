//! Application-level context providers and access helpers.

/// Provides authentication state and auth actions.
pub mod auth;
/// Provides viewport/mobile state derived from media queries.
pub mod mobile;
/// Provides notification state and helpers.
pub mod notification;

pub use auth::*;
pub use mobile::*;
pub use notification::*;
