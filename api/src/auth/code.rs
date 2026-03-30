//! Authorization code generation.
//!
//! Provides a helper for generating random numeric codes used during
//! email-based authentication flows.

use rand::RngExt;

/// Generates a random 6-digit authorization code.
///
/// The code is zero-padded so it is always exactly six characters
/// (e.g. `"004821"`).
///
/// # Returns
///
/// A [`String`] containing the 6-digit code.
pub fn generate() -> String {
    let code: u32 = rand::rng().random_range(0..1_000_000);
    format!("{code:06}")
}
