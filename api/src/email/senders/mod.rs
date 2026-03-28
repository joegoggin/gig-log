//! Specialized email sender implementations.
//!
//! Each submodule provides a sender struct that composes and delivers
//! emails for a specific feature area of the application.
//!
//! # Modules
//!
//! - [`auth`] — Authentication-related emails (verification, password reset, etc.).

pub mod auth;
