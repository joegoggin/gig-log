//! Authorization code database operations.
//!
//! Provides [`AuthCodeRepo`] for inserting, finding, and consuming
//! authorization codes stored in the `auth_codes` table.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::core::error::ApiResult;

/// The category of an authorization code.
///
/// Maps to the PostgreSQL `code_type` enum.
#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "code_type", rename_all = "snake_case")]
pub enum AuthCodeType {
    /// Confirms a newly registered email address.
    EmailVerification,
    /// Resets a forgotten password.
    PasswordReset,
    /// Confirms a request to change email addresses.
    EmailChange,
    /// Confirms a request to change the current password.
    PasswordChange,
}

/// A row from the `auth_codes` table.
#[derive(Debug, FromRow)]
pub struct AuthCodeRecord {
    /// Unique identifier for the auth code record.
    pub id: Uuid,
    /// The user this code belongs to.
    pub user_id: Uuid,
    /// The authorization code string.
    pub code: String,
    /// The category of operation this code authorizes.
    pub code_type: AuthCodeType,
    /// The new email address, present only for [`AuthCodeType::EmailChange`] codes.
    pub new_email: Option<String>,
    /// When this code expires.
    pub expires_at: DateTime<Utc>,
    /// Whether this code has already been consumed.
    pub used: bool,
}

/// Repository for authorization code database operations.
pub struct AuthCodeRepo;

impl AuthCodeRepo {
    /// Inserts a new authorization code.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user the code belongs to.
    /// * `code` — The authorization code string.
    /// * `code_type` — The category of operation this code authorizes.
    /// * `expires_at` — When the code should expire.
    /// * `new_email` — The new email address, required for [`AuthCodeType::EmailChange`] codes.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the insert query fails.
    pub async fn insert_code(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        code: &str,
        code_type: AuthCodeType,
        expires_at: DateTime<Utc>,
        new_email: Option<&str>,
    ) -> ApiResult<()> {
        sqlx::query!(
            r#"
        INSERT INTO auth_codes (user_id, code, code_type, expires_at, new_email)
        VALUES ($1, $2, $3, $4, $5)
        "#,
            user_id,
            code,
            code_type as AuthCodeType,
            expires_at,
            new_email,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Finds a valid (unused and non-expired) authorization code.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `code` — The authorization code string to look up.
    /// * `code_type` — The expected code category.
    ///
    /// # Returns
    ///
    /// The matching [`AuthCodeRecord`].
    ///
    /// # Errors
    ///
    /// Returns an error if no valid code is found.
    pub async fn find_valid_code(
        pool: &Pool<Postgres>,
        code: &str,
        code_type: AuthCodeType,
    ) -> ApiResult<AuthCodeRecord> {
        let auth_code = sqlx::query_as!(
            AuthCodeRecord,
            r#"
        SELECT id, user_id, code, code_type AS "code_type: AuthCodeType", new_email, expires_at, used
        FROM auth_codes
        WHERE code = $1
          AND code_type = $2
          AND used = FALSE
          AND expires_at > NOW()
        "#,
            code,
            code_type as AuthCodeType,
        )
        .fetch_one(pool)
        .await?;

        Ok(auth_code)
    }

    /// Marks an authorization code as used.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `code_id` — The UUID of the auth code record to consume.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn mark_used(pool: &Pool<Postgres>, code_id: Uuid) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE auth_codes
        SET used = TRUE
        WHERE id = $1
        "#,
            code_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
