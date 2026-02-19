//! Work session handlers for starting, pausing, resuming, completing, and
//! retrieving active work sessions.
//!
//! This module provides HTTP handlers for work session timer management
//! endpoints scoped to the authenticated user.
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for work session endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration.
pub use handlers::{
    complete_work_session, get_active_work_session, list_work_sessions_for_job,
    pause_work_session, resume_work_session, start_work_session,
};
