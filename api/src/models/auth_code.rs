use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "auth_code_type", rename_all = "snake_case")]
pub enum AuthCodeType {
    EmailConfirmation,
    PasswordReset,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct AuthCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code_hash: String,
    pub code_type: AuthCodeType,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}
