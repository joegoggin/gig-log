//! Password hashing and verification helpers.
//!
//! This module wraps Argon2 password hashing and verification with API-level
//! error mapping so authentication handlers can fail consistently.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::core::error::ApiError;

/// Hashes a plain-text password using Argon2 and a random salt.
///
/// # Arguments
///
/// - `password` - Plain-text password provided by a user
///
/// # Errors
///
/// Returns [`ApiError::InternalError`] if hashing fails.
pub fn hash_password(password: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| ApiError::InternalError("Failed to hash password".to_string()))?
        .to_string();

    Ok(password_hash)
}

/// Verifies a plain-text password against a stored Argon2 hash.
///
/// # Arguments
///
/// - `password` - Plain-text password provided by a user
/// - `password_hash` - Stored Argon2 password hash
///
/// # Errors
///
/// Returns [`ApiError::InternalError`] if the stored hash format is invalid.
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, ApiError> {
    let parsed_hash = PasswordHash::new(password_hash)
        .map_err(|_| ApiError::InternalError("Invalid password hash format".to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};
    use crate::core::error::ApiError;

    #[test]
    // Verifies a hashed password validates successfully against its original input.
    fn hash_password_and_verify_password_round_trip() {
        let password = "my-very-strong-password";
        let hash = hash_password(password).expect("password should hash");

        let is_valid = verify_password(password, &hash).expect("verification should succeed");

        assert!(is_valid);
    }

    #[test]
    // Verifies password verification rejects an incorrect password.
    fn verify_password_rejects_wrong_password() {
        let hash = hash_password("correct-password").expect("password should hash");

        let is_valid = verify_password("wrong-password", &hash).expect("verification should run");

        assert!(!is_valid);
    }

    #[test]
    // Verifies invalid stored hash formats are surfaced as internal errors.
    fn verify_password_returns_error_for_invalid_hash_format() {
        let result = verify_password("password", "not-a-valid-hash");

        assert!(matches!(result, Err(ApiError::InternalError(_))));
    }
}
