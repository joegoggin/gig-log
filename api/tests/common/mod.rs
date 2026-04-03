//! Shared integration-test helpers for the API crate.
//!
//! This module centralizes setup used by API integration tests, including a
//! deterministic configuration, test database connections with migrations,
//! application state construction, router construction, and table cleanup.
//!
//! # Environment
//!
//! - `TEST_DATABASE_URL` — Required PostgreSQL connection URL for tests.
//! - `TEST_RESEND_BASE_URL` — Optional email API base URL (wiremock).

#![allow(dead_code)]

mod helpers;

#[allow(unused_imports)]
pub use helpers::{
    cleanup_auth_tables, test_app_state, test_config, test_db_pool, test_router,
};
