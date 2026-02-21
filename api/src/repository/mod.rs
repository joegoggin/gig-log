//! Database repository layer for API domains.
//!
//! This module groups SQLx-backed data access helpers that keep persistence
//! logic out of HTTP handlers and service code.
//!
//! # Modules
//!
//! - [`appearance`] - Custom palette and active appearance preference queries
//! - [`auth`] - User, authentication code, and refresh token queries
//! - [`companies`] - Company CRUD queries scoped to authenticated users
//! - [`jobs`] - Job CRUD queries scoped to authenticated users
//! - [`payments`] - Payment CRUD queries scoped to authenticated users
//! - [`work_sessions`] - Work session CRUD queries scoped to authenticated users

pub mod appearance;
pub mod auth;
pub mod companies;
pub mod jobs;
pub mod payments;
pub mod work_sessions;
