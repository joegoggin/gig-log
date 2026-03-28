//! Password hashing and verification.
//!
//! Provides [`PasswordUtil`] for securely hashing passwords with
//! Argon2 and verifying plaintext passwords against stored hashes.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::core::error::{ApiErrorResponse, ApiResult};

/// Utility for hashing and verifying passwords with Argon2.
pub struct PasswordUtil;

impl PasswordUtil {
    /// Hashes a plaintext password using Argon2 with a random salt.
    ///
    /// # Arguments
    ///
    /// * `password` — The plaintext password to hash.
    ///
    /// # Returns
    ///
    /// The Argon2 hash as a [`String`] in PHC format.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`] if hashing
    /// fails.
    pub fn hash_password(password: &str) -> ApiResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| ApiErrorResponse::InternalServerError(error.to_string()))?;
        Ok(hash.to_string())
    }

    /// Verifies a plaintext password against an Argon2 hash.
    ///
    /// # Arguments
    ///
    /// * `password` — The plaintext password to check.
    /// * `hash` — The stored Argon2 hash in PHC format.
    ///
    /// # Returns
    ///
    /// `true` if the password matches the hash, `false` otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::InternalServerError`] if the hash
    /// string cannot be parsed.
    pub fn verify_password(password: &str, hash: &str) -> ApiResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|error| ApiErrorResponse::InternalServerError(error.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
