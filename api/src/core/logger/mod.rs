//! Structured logging and HTTP request/response logging.
//!
//! This module provides [`Logger`], the application-wide log backend, and
//! [`HttpLoggingConfig`], which controls the Axum middleware that logs HTTP
//! traffic. Sensitive values in headers and JSON bodies are automatically
//! redacted before output.

mod backend;
mod formatting;
mod middleware;
mod redaction;

pub use backend::Logger;
pub use middleware::HttpLoggingConfig;
