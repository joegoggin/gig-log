//! HTTP route handlers for the API.
//!
//! This module organizes all route handlers by domain:
//!
//! - [`auth`] - Authentication routes (sign-up, login, logout, password reset)
//! - [`companies`] - Company CRUD endpoints for authenticated users
//! - [`health`] - Health check endpoint for monitoring
//! - [`jobs`] - Job CRUD endpoints for authenticated users
//! - [`payments`] - Payment CRUD endpoints for authenticated users

pub mod auth;
pub mod companies;
pub mod health;
pub mod jobs;
pub mod payments;
