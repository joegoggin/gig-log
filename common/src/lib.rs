//! Shared types and utilities for the GigLog application.
//!
//! The `gig-log-common` crate provides the data models and validation logic
//! shared across GigLog services. It is the single source of truth for domain
//! entities, API request/response payloads, and error types.
//!
//! # Modules
//!
//! - [`models`] — Domain entities (users, companies, jobs, payments, work
//!   sessions, appearance preferences), request/response payloads, and
//!   structured error types.
//! - [`validators`] — Custom validation functions for cross-field constraints
//!   such as password confirmation matching.
//! - [`logging`] — Shared logging utilities and re-exports of the [`log`] facade
//!   macros.
//!
//! # Feature flags
//!
//! - **`validation`** — Enables request payload validation. When active, request
//!   structs derive [`validator::Validate`] and custom validator functions in the
//!   [`validators`] module are compiled. Disabled by default.

/// Shared logging utilities and re-exports.
pub mod logging;
/// Shared data models used across the GigLog application.
pub mod models;
/// Validation functions for request payloads.
pub mod validators;
