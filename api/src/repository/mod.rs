//! Database repository layer for API domains.
//!
//! This module groups SQLx-backed data access helpers that keep persistence
//! logic out of HTTP handlers and service code.
//!
//! # Modules
//!
//! - [`auth`] - User, authentication code, and refresh token queries

pub mod auth;
