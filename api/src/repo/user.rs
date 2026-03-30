//! User account database operations.
//!
//! Provides [`UserRepo`] for creating, querying, and updating user
//! records in the `users` table.

use sqlx::{Pool, Postgres};
use uuid::Uuid;

use gig_log_common::models::user::User;

use crate::core::error::ApiResult;

/// Repository for user account database operations.
pub struct UserRepo;

impl UserRepo {
    /// Inserts a new user record.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `first_name` — The user's first name.
    /// * `last_name` — The user's last name.
    /// * `email` — The user's email address.
    /// * `password_hash` — The Argon2 hash of the user's password.
    ///
    /// # Returns
    ///
    /// The newly created [`User`].
    ///
    /// # Errors
    ///
    /// Returns an error if the insert fails (e.g., duplicate email).
    pub async fn insert_user(
        pool: &Pool<Postgres>,
        first_name: &str,
        last_name: &str,
        email: &str,
        password_hash: &str,
    ) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
        INSERT INTO users (first_name, last_name, email, hashed_password)
        VALUES ($1, $2, $3, $4)
        RETURNING id, first_name, last_name, email, email_confirmed, created_at, updated_at
        "#,
            first_name,
            last_name,
            email,
            password_hash,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Finds a user by email address.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `email` — The email address to search for.
    ///
    /// # Returns
    ///
    /// The [`User`] matching the given email.
    ///
    /// # Errors
    ///
    /// Returns an error if no user with the given email exists.
    pub async fn find_user_by_email(pool: &Pool<Postgres>, email: &str) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
        SELECT id, first_name, last_name, email, email_confirmed, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
            email,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Finds a user by their unique identifier.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `id` — The user's UUID.
    ///
    /// # Returns
    ///
    /// The [`User`] matching the given ID.
    ///
    /// # Errors
    ///
    /// Returns an error if no user with the given ID exists.
    pub async fn find_user_by_id(pool: &Pool<Postgres>, id: Uuid) -> ApiResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
        SELECT id, first_name, last_name, email, email_confirmed, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
            id,
        )
        .fetch_one(pool)
        .await?;

        Ok(user)
    }

    /// Retrieves the stored password hash for a user.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user's UUID.
    ///
    /// # Returns
    ///
    /// The Argon2 password hash as a [`String`].
    ///
    /// # Errors
    ///
    /// Returns an error if no user with the given ID exists.
    pub async fn get_password_hash(pool: &Pool<Postgres>, user_id: Uuid) -> ApiResult<String> {
        let hashed_password = sqlx::query_scalar!(
            r#"
        SELECT hashed_password
        FROM users
        WHERE id = $1
        "#,
            user_id,
        )
        .fetch_one(pool)
        .await?;

        Ok(hashed_password)
    }

    /// Updates a user's password hash.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user's UUID.
    /// * `password_hash` — The new Argon2 password hash.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn update_password(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        password_hash: &str,
    ) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE users
        SET hashed_password = $1, updated_at = NOW()
        WHERE id = $2
        "#,
            password_hash,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Marks a user's email as confirmed.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user's UUID.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn confirm_email(pool: &Pool<Postgres>, user_id: Uuid) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE users
        SET email_confirmed = TRUE, updated_at = NOW()
        WHERE id = $1
        "#,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Updates a user's email address and marks it as confirmed.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user's UUID.
    /// * `new_email` — The new email address.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn update_email_and_confirm(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        new_email: &str,
    ) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE users
        SET email = $1, email_confirmed = TRUE, updated_at = NOW()
        WHERE id = $2
        "#,
            new_email,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Marks a user's email as unconfirmed.
    ///
    /// # Arguments
    ///
    /// * `pool` — The database connection pool.
    /// * `user_id` — The user's UUID.
    ///
    /// # Returns
    ///
    /// `()` on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the update query fails.
    pub async fn set_email_unconfirmed(pool: &Pool<Postgres>, user_id: Uuid) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE users
        SET email_confirmed = FALSE, updated_at = NOW()
        WHERE id = $1
        "#,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
