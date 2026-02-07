//! Health check endpoint for API monitoring.
//!
//! Provides a simple endpoint to verify the API is running and responsive.
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler for the health check endpoint

pub mod handlers;

// Re-export handler at module level for easy route registration
pub use handlers::health_check;
