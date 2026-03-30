use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The current status of a work session. Serialized as `snake_case`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkSessionStatus {
    /// The session is actively tracking time.
    Active,
    /// The session is temporarily paused.
    Paused,
    /// The session has been completed.
    Completed,
}

/// A time-tracking work session for a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSession {
    /// Unique identifier for the work session.
    pub id: Uuid,
    /// The user who owns this work session.
    pub user_id: Uuid,
    /// The job this work session is associated with.
    pub job_id: Uuid,
    /// When the work session started.
    pub start_time: DateTime<Utc>,
    /// When the work session ended. `None` if still in progress.
    pub end_time: Option<DateTime<Utc>>,
    /// Whether the session timer is currently running.
    pub is_running: bool,
    /// Total accumulated time spent paused, in seconds.
    pub accumulated_paused_duration: i64,
    /// When the session was last paused. `None` if not currently paused.
    pub paused_at: Option<DateTime<Utc>>,
    /// Manually reported time in seconds, if provided.
    pub time_reported: Option<i64>,
    /// Timestamp when the work session was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the work session was last updated.
    pub updated_at: DateTime<Utc>,
}
