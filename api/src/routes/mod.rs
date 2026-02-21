//! HTTP route handlers for the API.
//!
//! This module organizes all route handlers by domain:
//!
//! - [`appearance`] - User appearance preferences and custom palette endpoints
//! - [`auth`] - Authentication routes (sign-up, login, logout, password reset, email change)
//! - [`companies`] - Company CRUD endpoints for authenticated users
//! - [`health`] - Health check endpoint for monitoring
//! - [`jobs`] - Job CRUD endpoints for authenticated users
//! - [`payments`] - Payment CRUD endpoints for authenticated users
//! - [`work_sessions`] - Work session timer management endpoints for authenticated users

pub mod appearance;
pub mod auth;
pub mod companies;
pub mod health;
pub mod jobs;
pub mod payments;
pub mod work_sessions;
