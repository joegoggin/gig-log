use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WorkSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub job_id: Uuid,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub is_running: bool,
    pub accumulated_paused_duration: i64, // Stored as Seconds (BigInt)
    pub paused_at: Option<DateTime<Utc>>,
    pub time_reported: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
