//! Database repository layer for the GigLog API.
//!
//! Each sub-module exposes a zero-sized marker struct (e.g., [`user::UserRepo`](crate::repo::user::UserRepo))
//! whose associated async functions execute SQLx queries against a PostgreSQL
//! connection pool and return [`ApiResult`](crate::core::error::ApiResult) values.
//!
//! # Modules
//!
//! - [`auth_code`](crate::repo::auth_code) — Authorization code storage and validation.
//! - [`refresh_token`](crate::repo::refresh_token) — Refresh token storage and revocation.
//! - [`user`](crate::repo::user) — User account CRUD operations.

pub mod auth_code;
pub mod refresh_token;
pub mod user;
