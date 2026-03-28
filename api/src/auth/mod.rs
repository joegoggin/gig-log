//! Authentication and authorization for the GigLog API.
//!
//! This module provides the building blocks for user authentication,
//! including JWT token management, password hashing, cookie handling,
//! and an Axum request extractor for identifying authenticated users.
//!
//! # Modules
//!
//! - [`code`] — Authorization code generation.
//! - [`cookies`] — Cookie construction and clearing for auth tokens.
//! - [`jwt`] — JWT token creation and validation.
//! - [`password`] — Password hashing and verification with Argon2.
//! - [`user`] — [`AuthUser`] Axum extractor for protected routes.

pub mod code;
pub mod cookies;
pub mod jwt;
pub mod password;
pub mod user;

pub use user::AuthUser;
