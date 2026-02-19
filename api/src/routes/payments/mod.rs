//! Payment handlers for listing, viewing, creating, updating, and deleting payments.
//!
//! This module provides HTTP handlers for payment management endpoints scoped
//! to the authenticated user.
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for payment endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration.
pub use handlers::{create_payment, delete_payment, get_payment, list_payments, update_payment};

// Re-export payload types used by validators and tests.
pub use payloads::{CreatePaymentRequest, UpdatePaymentRequest};
