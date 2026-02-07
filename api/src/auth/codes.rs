//! One-time authentication code utilities.
//!
//! This module supports short numeric code flows (for example email confirmation
//! and password reset) by generating six-digit codes, hashing codes for storage,
//! and verifying user input against stored hashes.

use rand::Rng;
use sha2::{Digest, Sha256};

/// Generates a random six-digit authentication code as a string.
pub fn generate_auth_code() -> String {
    let mut rng = rand::thread_rng();
    let code: u32 = rng.gen_range(100000..1000000);
    code.to_string()
}

/// Hashes an authentication code using SHA-256 and returns a hex-encoded digest.
///
/// # Arguments
///
/// - `code` - Plain-text authentication code to hash
pub fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    hex::encode(hasher.finalize())
}

/// Verifies a plain-text code against a previously hashed code.
///
/// Uses constant-time comparison to reduce timing side-channel leakage.
///
/// # Arguments
///
/// - `code` - User-provided plain-text code
/// - `hash` - Stored hex-encoded SHA-256 hash
pub fn verify_code(code: &str, hash: &str) -> bool {
    let code_hash = hash_code(code);
    constant_time_compare(&code_hash, hash)
}

/// Compares two strings in constant time when lengths match.
///
/// # Arguments
///
/// - `a` - First string
/// - `b` - Second string
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (byte_a, byte_b) in a.bytes().zip(b.bytes()) {
        result |= byte_a ^ byte_b;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::{generate_auth_code, hash_code, verify_code};

    #[test]
    // Verifies generated auth codes are always six numeric characters.
    fn generate_auth_code_returns_six_digit_numeric_value() {
        let code = generate_auth_code();

        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    // Verifies hashing is deterministic for the same auth code input.
    fn hash_code_is_deterministic_for_same_input() {
        let first = hash_code("123456");
        let second = hash_code("123456");

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    // Verifies code verification succeeds for matches and fails for mismatches.
    fn verify_code_accepts_matching_and_rejects_non_matching_codes() {
        let hash = hash_code("654321");

        assert!(verify_code("654321", &hash));
        assert!(!verify_code("111111", &hash));
    }
}
