use chrono::{DateTime, Utc};
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::core::error::ApiResult;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "auth_code_type", rename_all = "snake_case")]
pub enum AuthCodeType {
    EmailVerification,
    PasswordReset,
    EmailChange,
}

#[derive(Debug, FromRow)]
pub struct AuthCodeRecord {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code: String,
    pub code_type: AuthCodeType,
    pub new_email: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
}

pub struct AuthCodeRepo;

impl AuthCodeRepo {
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
