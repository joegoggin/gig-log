//! Company handlers for listing, viewing, creating, updating, and deleting companies.
//!
//! This module provides HTTP handlers for company management endpoints scoped
//! to the authenticated user.
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for company endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration.
pub use handlers::{create_company, delete_company, get_company, list_companies, update_company};

// Re-export payload types used by validators and tests.
pub use payloads::{CreateCompanyRequest, UpdateCompanyRequest};
