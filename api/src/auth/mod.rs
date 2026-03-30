//! Authentication and authorization for the GigLog API.
//!
//! This module provides the building blocks for user authentication,
//! including JWT token management, password hashing, cookie handling,
//! and an Axum request extractor for identifying authenticated users.
//!
//! # Modules
//!
//! - [`code`](crate::auth::code) — Authorization code generation.
//! - [`cookies`](crate::auth::cookies) — Cookie construction and clearing for auth tokens.
//! - [`jwt`](crate::auth::jwt) — JWT token creation and validation.
//! - [`password`](crate::auth::password) — Password hashing and verification with Argon2.
//! - [`user`](crate::auth::user) — [`AuthUser`](crate::auth::AuthUser) Axum extractor for protected routes.

pub mod code;
pub mod cookies;
pub mod jwt;
pub mod password;
pub mod user;

pub use user::AuthUser;
