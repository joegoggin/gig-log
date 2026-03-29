//! Frontend API client modules for backend communication.

/// Defines the low-level HTTP API client.
pub mod client;
/// Defines frontend API client error types.
pub mod error;
/// Defines grouped request runners by feature area.
pub mod requests;

pub use client::*;
pub use error::*;
pub use requests::auth::AuthRequestRunner;
