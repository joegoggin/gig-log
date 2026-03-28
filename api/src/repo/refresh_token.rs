//! Refresh token database operations.
//!
//! Provides [`RefreshTokenRepo`] for inserting, finding, and revoking
//! refresh tokens stored in the `refresh_tokens` table.

use chrono::{DateTime, Utc};
use log::warn;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::core::error::ApiResult;

/// A row from the `refresh_tokens` table.
#[derive(Debug, FromRow)]
pub struct RefreshTokenRecord {
    /// Unique identifier for the token record.
    pub id: Uuid,
    /// The user this token belongs to.
    pub user_id: Uuid,
    /// SHA-256 hash of the raw refresh token.
    pub token_hash: String,
    /// When this token expires.
    pub expires_at: DateTime<Utc>,
    /// Whether this token has been revoked.
    pub revoked: bool,
}

/// Repository for refresh token database operations.
pub struct RefreshTokenRepo;

impl RefreshTokenRepo {
    /// Inserts a new refresh token with a 30-day expiration.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user the token belongs to.
    /// * `token_hash` — The SHA-256 hash of the raw refresh token.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the insert query fails.
    pub async fn insert_token(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        token_hash: &str,
    ) -> ApiResult<()> {
        sqlx::query!(
            r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '30 days')
        "#,
            user_id,
            token_hash,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Finds a valid (non-revoked and non-expired) refresh token by its hash.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `token_hash` — The SHA-256 hash of the raw refresh token.
    ///
    /// # Returns
    ///
    /// The matching [`RefreshTokenRecord`].
    ///
    /// # Errors
    ///
    /// Returns an error if no valid token is found.
    pub async fn find_by_hash(
        pool: &Pool<Postgres>,
        token_hash: &str,
    ) -> ApiResult<RefreshTokenRecord> {
        let refresh_token = sqlx::query_as!(
            RefreshTokenRecord,
            r#"
        SELECT id, user_id, token_hash, expires_at, revoked
        FROM refresh_tokens
        WHERE token_hash = $1
          AND revoked = FALSE
          AND expires_at > NOW()
        "#,
            token_hash,
        )
        .fetch_one(pool)
        .await?;

        Ok(refresh_token)
    }

    /// Revokes a single refresh token by its hash.
    ///
    /// Logs a warning if no active token matches the given hash.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `token_hash` — The SHA-256 hash of the token to revoke.
    ///
    /// # Returns
    ///
    /// `true` if a token was revoked, `false` if no active token matched.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn revoke_token(pool: &Pool<Postgres>, token_hash: &str) -> ApiResult<bool> {
        let result = sqlx::query!(
            r#"
        UPDATE refresh_tokens
        SET revoked = TRUE
        WHERE token_hash = $1
          AND revoked = FALSE
        "#,
            token_hash,
        )
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            warn!("No active refresh token record matched provided hash during logout");

            return Ok(false);
        }

        Ok(true)
    }

    /// Revokes all active refresh tokens for a user.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user whose tokens should be revoked.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn revoke_all_for_user(pool: &Pool<Postgres>, user_id: Uuid) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE refresh_tokens
        SET revoked = TRUE
        WHERE user_id = $1 AND revoked = FALSE
        "#,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
