//! Refresh token model for JWT authentication token renewal.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A refresh token used to obtain new access tokens without re-authentication.
///
/// Tokens are hashed before storage and can be revoked to invalidate a user's
/// session. They expire after a configured time period.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct RefreshToken {
    /// Unique identifier for the refresh token.
    pub id: Uuid,
    /// The user this token belongs to.
    pub user_id: Uuid,
    /// Hashed version of the token for secure storage.
    pub token_hash: String,
    /// When this token expires and can no longer be used.
    pub expires_at: DateTime<Utc>,
    /// Whether this token has been revoked (e.g., on logout).
    pub revoked: bool,
    /// Timestamp when the token was created.
    pub created_at: DateTime<Utc>,
}
