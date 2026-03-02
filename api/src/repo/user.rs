use sqlx::{Pool, Postgres};
use uuid::Uuid;

use gig_log_common::models::user::User;

use crate::core::error::ApiResult;

pub struct UserRepo;

impl UserRepo {
    pub async fn insert_user(
        pool: &Pool<Postgres>,
        first_name: &str,
        last_name: &str,
        email: &str,
        password_hash: &str,
    ) -> ApiResult<User> {
        sqlx::query_as!(
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
        .await
        .map_err(|error| error.into())
    }

    pub async fn find_user_by_email(pool: &Pool<Postgres>, email: &str) -> ApiResult<User> {
        sqlx::query_as!(
            User,
            r#"
        SELECT id, first_name, last_name, email, email_confirmed, created_at, updated_at
        FROM users
        WHERE email = $1
        "#,
            email,
        )
        .fetch_one(pool)
        .await
        .map_err(|error| error.into())
    }

    pub async fn find_user_by_id(pool: &Pool<Postgres>, id: Uuid) -> ApiResult<User> {
        sqlx::query_as!(
            User,
            r#"
        SELECT id, first_name, last_name, email, email_confirmed, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
            id,
        )
        .fetch_one(pool)
        .await
        .map_err(|error| error.into())
    }

    pub async fn get_password_hash(pool: &Pool<Postgres>, user_id: Uuid) -> ApiResult<String> {
        sqlx::query_scalar!(
            r#"
        SELECT hashed_password
        FROM users
        WHERE id = $1
        "#,
            user_id,
        )
        .fetch_one(pool)
        .await
        .map_err(|error| error.into())
    }

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

    pub async fn update_email(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        new_email: &str,
    ) -> ApiResult<()> {
        sqlx::query!(
            r#"
        UPDATE users
        SET email = $1, email_confirmed = FALSE, updated_at = NOW()
        WHERE id = $2
        "#,
            new_email,
            user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
