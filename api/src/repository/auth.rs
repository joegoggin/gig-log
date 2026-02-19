//! Authentication-focused repository operations.
//!
//! This module centralizes SQL queries used by authentication flows, including
//! user lookup and creation, auth code lifecycle management, and refresh token
//! persistence operations.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::auth_code::AuthCodeType;

/// User fields required for login verification.
pub struct UserForLogin {
    /// Unique user identifier.
    pub id: Uuid,
    /// User email address.
    pub email: String,
    /// Stored password hash used for password verification.
    pub hashed_password: String,
    /// Whether the user has confirmed their email.
    pub email_confirmed: bool,
}

/// User fields required for email confirmation checks.
pub struct UserForConfirmation {
    /// Unique user identifier.
    pub id: Uuid,
    /// Whether the user has already confirmed their email.
    pub email_confirmed: bool,
}

/// User fields required for password reset initiation.
pub struct UserForPasswordReset {
    /// Unique user identifier.
    pub id: Uuid,
    /// User first name for personalization in reset communications.
    pub first_name: String,
}

/// User fields required when verifying forgot-password codes.
pub struct UserForVerification {
    /// Unique user identifier.
    pub id: Uuid,
    /// User email address.
    pub email: String,
}

/// User fields required for refresh-token rotation.
pub struct UserForTokenRefresh {
    /// Unique user identifier.
    pub id: Uuid,
    /// User email address.
    pub email: String,
}

/// Public user profile returned for authenticated sessions.
#[derive(Debug, Serialize)]
pub struct CurrentUser {
    /// Unique user identifier.
    pub id: Uuid,
    /// User first name.
    pub first_name: String,
    /// User last name.
    pub last_name: String,
    /// User email address.
    pub email: String,
    /// Whether the user has confirmed their email.
    pub email_confirmed: bool,
    /// Timestamp when the user record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the user record was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Auth code record used during code verification.
pub struct ValidAuthCode {
    /// Unique auth code identifier.
    pub id: Uuid,
    /// Stored auth code hash for secure comparison.
    pub code_hash: String,
}

/// Repository methods for authentication-related persistence.
pub struct AuthRepo;

impl AuthRepo {
    /// Checks whether a user already exists for an email address.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `email` - Email address to check
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn check_email_exists(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT id FROM users WHERE LOWER(email) = LOWER($1)"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.is_some())
    }

    /// Finds user credentials and account state for login.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `email` - Email address to look up
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_user_for_login(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForLogin>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForLogin,
            r#"SELECT id, email, hashed_password, email_confirmed FROM users WHERE LOWER(email) = LOWER($1)"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Finds user data required to confirm an email address.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `email` - Email address to look up
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_user_for_confirmation(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForConfirmation>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForConfirmation,
            r#"SELECT id, email_confirmed FROM users WHERE LOWER(email) = LOWER($1)"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Finds user data required to start a password reset flow.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `email` - Email address to look up
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_user_for_password_reset(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForPasswordReset>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForPasswordReset,
            r#"SELECT id, first_name FROM users WHERE LOWER(email) = LOWER($1)"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Finds user data needed to verify forgot-password codes.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `email` - Email address to look up
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_user_for_verification(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForVerification>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForVerification,
            r#"SELECT id, email FROM users WHERE LOWER(email) = LOWER($1)"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Finds user data needed to rotate refresh/access tokens.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User identifier to look up
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_user_for_token_refresh(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> Result<Option<UserForTokenRefresh>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForTokenRefresh,
            r#"SELECT id, email FROM users WHERE id = $1"#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Finds a user by ID for the authenticated "current user" response.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User identifier to look up
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_user_by_id(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> Result<Option<CurrentUser>, sqlx::Error> {
        let result = sqlx::query_as!(
            CurrentUser,
            r#"
        SELECT id, first_name, last_name, email, email_confirmed, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Creates a new user account and returns the inserted user ID.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `first_name` - User first name
    /// - `last_name` - User last name
    /// - `email` - User email address
    /// - `hashed_password` - Password hash to persist
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_user(
        pool: &Pool<Postgres>,
        first_name: &str,
        last_name: &str,
        email: &str,
        hashed_password: &str,
    ) -> Result<Uuid, sqlx::Error> {
        let user_id = sqlx::query_scalar!(
            r#"
        INSERT INTO users (first_name, last_name, email, hashed_password, email_confirmed)
        VALUES ($1, $2, $3, $4, false)
        RETURNING id
        "#,
            first_name,
            last_name,
            email,
            hashed_password
        )
        .fetch_one(pool)
        .await?;

        Ok(user_id)
    }

    /// Marks a user's email as confirmed within an existing transaction.
    ///
    /// # Arguments
    ///
    /// - `tx` - Active database transaction
    /// - `user_id` - User identifier to update
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn confirm_user_email(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE users SET email_confirmed = true, updated_at = NOW() WHERE id = $1"#,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Updates a user's password hash within an existing transaction.
    ///
    /// # Arguments
    ///
    /// - `tx` - Active database transaction
    /// - `user_id` - User identifier to update
    /// - `hashed_password` - New password hash
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn update_user_password(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        user_id: Uuid,
        hashed_password: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE users SET hashed_password = $1, updated_at = NOW() WHERE id = $2"#,
            hashed_password,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Stores a hashed authentication code for a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User that owns the code
    /// - `code_hash` - Hashed code value
    /// - `code_type` - Authentication code purpose
    /// - `expires_at` - Expiration timestamp for the code
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_auth_code(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        code_hash: &str,
        code_type: AuthCodeType,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        INSERT INTO auth_codes (user_id, code_hash, code_type, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
            user_id,
            code_hash,
            code_type as AuthCodeType,
            expires_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Finds the most recent unexpired and unused auth code for a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User that owns the auth code
    /// - `code_type` - Authentication code purpose
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_valid_auth_code(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        code_type: AuthCodeType,
    ) -> Result<Option<ValidAuthCode>, sqlx::Error> {
        let result = sqlx::query_as!(
            ValidAuthCode,
            r#"
        SELECT id, code_hash
        FROM auth_codes
        WHERE user_id = $1
          AND code_type = $2
          AND used = false
          AND expires_at > NOW()
        ORDER BY created_at DESC
        LIMIT 1
        "#,
            user_id,
            code_type as AuthCodeType
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    /// Marks an auth code as used within an existing transaction.
    ///
    /// # Arguments
    ///
    /// - `tx` - Active database transaction
    /// - `code_id` - Auth code identifier to update
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn mark_auth_code_used(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        code_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE auth_codes SET used = true WHERE id = $1"#,
            code_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Marks an auth code as used without requiring a transaction.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `code_id` - Auth code identifier to update
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn mark_auth_code_used_without_tx(
        pool: &Pool<Postgres>,
        code_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE auth_codes SET used = true WHERE id = $1"#,
            code_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Invalidates all active password reset codes for a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User whose password reset codes should be invalidated
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn invalidate_password_reset_codes(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        UPDATE auth_codes
        SET used = true
        WHERE user_id = $1 AND code_type = 'password_reset' AND used = false
        "#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Creates a refresh token record for a user session.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User that owns the refresh token
    /// - `token_hash` - Hashed refresh token value
    /// - `expires_at` - Expiration timestamp for the token
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_refresh_token(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
            user_id,
            token_hash,
            expires_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Creates a refresh token record for a user session within a transaction.
    ///
    /// # Arguments
    ///
    /// - `tx` - Active database transaction
    /// - `user_id` - User that owns the refresh token
    /// - `token_hash` - Hashed refresh token value
    /// - `expires_at` - Expiration timestamp for the token
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_refresh_token_in_tx(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
        INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, $3)
        "#,
            user_id,
            token_hash,
            expires_at
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Checks whether a refresh token is still active for a user.
    ///
    /// A token is considered active when it belongs to the user, is not
    /// revoked, and has not expired.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - User that owns the refresh token
    /// - `token_hash` - Hashed refresh token value to check
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn is_refresh_token_active(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
        SELECT COUNT(*)::bigint
        FROM refresh_tokens
        WHERE user_id = $1
          AND token_hash = $2
          AND revoked = false
          AND expires_at > NOW()
        "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Atomically consumes an active refresh token inside a transaction.
    ///
    /// A token is consumed when it belongs to the user, is not revoked, has
    /// not expired, and is marked revoked by this operation.
    ///
    /// # Arguments
    ///
    /// - `tx` - Active database transaction
    /// - `user_id` - User that owns the refresh token
    /// - `token_hash` - Hashed refresh token value to consume
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update query fails.
    pub async fn consume_active_refresh_token(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        user_id: Uuid,
        token_hash: &str,
    ) -> Result<bool, sqlx::Error> {
        let consumed = sqlx::query!(
            r#"
        UPDATE refresh_tokens
        SET revoked = true
        WHERE user_id = $1
          AND token_hash = $2
          AND revoked = false
          AND expires_at > NOW()
        RETURNING id
        "#,
            user_id,
            token_hash
        )
        .fetch_optional(&mut **tx)
        .await?;

        Ok(consumed.is_some())
    }

    /// Revokes a refresh token by its hashed token value.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `token_hash` - Hashed refresh token value to revoke
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn revoke_refresh_token(
        pool: &Pool<Postgres>,
        token_hash: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE refresh_tokens SET revoked = true WHERE token_hash = $1"#,
            token_hash
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Revokes all refresh tokens for a user within an existing transaction.
    ///
    /// # Arguments
    ///
    /// - `tx` - Active database transaction
    /// - `user_id` - User whose refresh tokens should be revoked
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn revoke_all_user_refresh_tokens(
        tx: &mut sqlx::Transaction<'_, Postgres>,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE refresh_tokens SET revoked = true WHERE user_id = $1"#,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
