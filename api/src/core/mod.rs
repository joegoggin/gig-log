//! Core application bootstrap and infrastructure modules.
//!
//! This module groups the pieces required to initialize and run the API:
//!
//! - [`app`] - Top-level startup orchestration
//! - [`app_state`] - Shared runtime dependencies for handlers
//! - [`config`] - HTTP route registration
//! - [`mod@env`] - Environment variable loading and validation
//! - [`error`] - Shared API error types and HTTP error responses
//! - [`server`] - Actix server and middleware setup

pub mod app;
pub mod app_state;
pub mod config;
pub mod env;
pub mod error;
pub mod server;
