//! External service integrations used by API handlers.
//!
//! This module contains wrappers around third-party services so business logic
//! can depend on a small, testable interface.
//!
//! - [`email`] - Transactional email delivery via Resend

pub mod email;
