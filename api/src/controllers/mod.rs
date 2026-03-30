//! HTTP request handlers for the GigLog API.
//!
//! Each sub-module groups the Axum handler functions for a related set
//! of routes and exposes a controller struct whose methods are wired
//! to the application router.
//!
//! # Modules
//!
//! - [`auth`](crate::controllers::auth) — Authentication and account management endpoints.
//! - [`health`](crate::controllers::health) — Health check endpoints.

pub mod auth;
pub mod health;
