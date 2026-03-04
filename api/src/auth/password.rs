use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

use crate::core::error::{ApiErrorResponse, ApiResult};

pub struct PasswordUtil;

impl PasswordUtil {
    pub fn hash_password(password: &str) -> ApiResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|error| ApiErrorResponse::InternalServerError(error.to_string()))?;
        Ok(hash.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> ApiResult<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|error| ApiErrorResponse::InternalServerError(error.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
