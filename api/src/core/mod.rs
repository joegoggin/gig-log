//! Core infrastructure for the API server.
//!
//! This module contains the server entry point, configuration, error handling,
//! and logging setup.
//!
//! # Modules
//!
//! - [`app`](crate::core::app) — Application entry point and bootstrap sequence.
//! - [`config`](crate::core::config) — Environment-based configuration.
//! - [`error`](crate::core::error) — API error types and HTTP response conversions.
//! - [`logger`](crate::core::logger) — Structured logging setup.

pub mod app;
pub mod config;
pub mod error;
pub mod logger;
