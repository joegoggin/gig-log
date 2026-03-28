//! Route definitions for the GigLog API.
//!
//! This module organizes the application's HTTP routes into sub-modules,
//! each responsible for a distinct area of functionality. The top-level
//! [`app::AppRouter`] assembles all nested routers, applies shared
//! middleware, and binds the application state.
//!
//! # Modules
//!
//! - [`app`] — Application router, shared state, and middleware configuration.
//! - [`auth`] — Authentication and account management routes.
//! - [`health`] — Health check routes.

pub mod app;
pub mod auth;
pub mod health;
