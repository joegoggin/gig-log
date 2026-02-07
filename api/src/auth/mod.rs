//! Authentication and authorization helpers used across the API.
//!
//! This module provides reusable utilities for:
//!
//! - [`codes`] - Numeric authentication code generation and verification helpers
//! - [`cookies`] - Secure auth cookie construction and clearing
//! - [`jwt`] - JWT claim types and token encode/decode helpers
//! - [`middleware`] - Request extractor for authenticated users
//! - [`password`] - Password hashing and verification

pub mod codes;
pub mod cookies;
pub mod jwt;
pub mod middleware;
pub mod password;
