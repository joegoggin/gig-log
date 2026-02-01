//! Database models representing the core domain entities of the application.
//!
//! This module contains all SQLx-compatible structs that map to database tables,
//! including users, companies, jobs, payments, work sessions, and authentication-related models.

pub mod auth_code;
pub mod company;
pub mod job;
pub mod payment;
pub mod refresh_token;
pub mod user;
pub mod work_session;
