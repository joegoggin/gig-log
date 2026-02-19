//! Job handlers for listing, viewing, creating, updating, and deleting jobs.
//!
//! This module provides HTTP handlers for job management endpoints scoped
//! to the authenticated user.
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for job endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration.
pub use handlers::{create_job, delete_job, get_job, list_jobs, update_job};

// Re-export payload types used by validators and tests.
pub use payloads::{CreateJobRequest, UpdateJobRequest};
