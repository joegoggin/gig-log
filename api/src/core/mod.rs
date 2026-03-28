//! Core infrastructure for the API server.
//!
//! This module contains the server entry point, configuration, error handling,
//! and logging setup.
//!
//! - [`app`] — Application entry point and bootstrap sequence.
//! - [`config`] — Environment-based configuration.
//! - [`error`] — API error types and HTTP response conversions.
//! - [`logger`] — Structured logging setup.

pub mod app;
pub mod config;
pub mod error;
pub mod logger;
