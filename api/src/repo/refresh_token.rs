use chrono::{DateTime, Utc};
use log::error;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::core::error::ApiResult;

#[derive(Debug, FromRow)]
pub struct RefreshTokenRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
}

pub struct RefreshTokenRepo;

impl RefreshTokenRepo {
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
            error!("No active refresh token record matched provided hash during logout");

            return Ok(false);
        }

        Ok(true)
    }

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
