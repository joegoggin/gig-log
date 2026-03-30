//! Validation functions for request payloads.
//!
//! These validators are used with the `validator` crate and are only available
//! when the `"validation"` feature is enabled. They enforce cross-field
//! constraints (e.g., password confirmation matching) that cannot be expressed
//! with field-level derive attributes alone.

/// User-related validation functions.
pub mod user;
