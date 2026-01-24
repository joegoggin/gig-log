use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::auth_code::AuthCodeType;

pub struct UserForLogin {
    pub id: Uuid,
    pub email: String,
    pub hashed_password: String,
    pub email_confirmed: bool,
}

pub struct UserForConfirmation {
    pub id: Uuid,
    pub email_confirmed: bool,
}

pub struct UserForPasswordReset {
    pub id: Uuid,
    pub first_name: String,
}

pub struct UserForVerification {
    pub id: Uuid,
    pub email: String,
}

pub struct ValidAuthCode {
    pub id: Uuid,
    pub code_hash: String,
}

pub struct AuthRepo;

impl AuthRepo {
    pub async fn check_email_exists(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(r#"SELECT id FROM users WHERE email = $1"#, email)
            .fetch_optional(pool)
            .await?;

        Ok(result.is_some())
    }

    pub async fn find_user_for_login(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForLogin>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForLogin,
            r#"SELECT id, email, hashed_password, email_confirmed FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn find_user_for_confirmation(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForConfirmation>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForConfirmation,
            r#"SELECT id, email_confirmed FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn find_user_for_password_reset(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForPasswordReset>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForPasswordReset,
            r#"SELECT id, first_name FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

    pub async fn find_user_for_verification(
        pool: &Pool<Postgres>,
        email: &str,
    ) -> Result<Option<UserForVerification>, sqlx::Error> {
        let result = sqlx::query_as!(
            UserForVerification,
            r#"SELECT id, email FROM users WHERE email = $1"#,
            email
        )
        .fetch_optional(pool)
        .await?;

        Ok(result)
    }

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

    // Refresh token operations

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
