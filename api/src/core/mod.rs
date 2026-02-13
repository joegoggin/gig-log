//! Core application bootstrap and infrastructure modules.
//!
//! This module groups the pieces required to initialize and run the API:
//!
//! - [`app`] - Top-level startup orchestration
//! - [`app_state`] - Shared runtime dependencies for handlers
//! - [`config`] - HTTP route registration
//! - [`docker`] - Development Docker Compose auto-start checks
//! - [`mod@env`] - Environment variable loading and validation
//! - [`error`] - Shared API error types and HTTP error responses
//! - [`logger`] - Custom logging and HTTP request/response logging middleware
//! - [`server`] - Actix server and middleware setup

pub mod app;
pub mod app_state;
pub mod config;
pub mod docker;
pub mod env;
pub mod error;
pub mod logger;
pub mod server;
