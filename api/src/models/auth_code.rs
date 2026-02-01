//! Authentication code model for email confirmation and password reset flows.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

/// The type of authentication code, determining its purpose and handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "auth_code_type", rename_all = "snake_case")]
pub enum AuthCodeType {
    /// Code sent to verify a user's email address during registration.
    EmailConfirmation,
    /// Code sent to allow a user to reset their password.
    PasswordReset,
}

/// A time-limited authentication code used for email confirmation or password reset.
///
/// Codes are hashed before storage and can only be used once. They expire after
/// a configured time period.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct AuthCode {
    /// Unique identifier for the auth code.
    pub id: Uuid,
    /// The user this code was issued to.
    pub user_id: Uuid,
    /// Hashed version of the code for secure storage.
    pub code_hash: String,
    /// The purpose of this authentication code.
    pub code_type: AuthCodeType,
    /// When this code expires and can no longer be used.
    pub expires_at: DateTime<Utc>,
    /// Whether this code has already been used.
    pub used: bool,
    /// Timestamp when the code was created.
    pub created_at: DateTime<Utc>,
}
