//! Authentication handlers for user registration, login, and password management.
//!
//! This module provides HTTP handlers for all authentication-related endpoints:
//! - User registration and email confirmation
//! - Login and logout with JWT tokens stored in HTTP-only cookies
//! - Password reset flow (forgot password, verify code, set new password)
//! - Current user retrieval for authenticated sessions
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for authentication endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration
pub use handlers::{
    confirm_email, current_user, forgot_password, log_in, log_out, set_password, sign_up,
    verify_forgot_password,
};

// Re-export payload types that are used by other modules
pub use payloads::{SetPasswordRequest, SignUpRequest};
