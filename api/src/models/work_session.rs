//! Work session model for tracking time spent on jobs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a time-tracked work session for a specific job.
///
/// Sessions support start/stop/pause functionality and track total time
/// worked excluding paused periods.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct WorkSession {
    /// Unique identifier for the work session.
    pub id: Uuid,
    /// The user who owns this work session.
    pub user_id: Uuid,
    /// The job this session is tracking time for.
    pub job_id: Uuid,
    /// When the session was started.
    pub start_time: Option<DateTime<Utc>>,
    /// When the session was stopped/completed.
    pub end_time: Option<DateTime<Utc>>,
    /// Whether the session timer is currently running.
    pub is_running: bool,
    /// Total time spent paused, in seconds.
    pub accumulated_paused_duration: i64,
    /// When the session was paused (if currently paused).
    pub paused_at: Option<DateTime<Utc>>,
    /// Whether this session's time has been reported/submitted.
    pub time_reported: bool,
    /// Timestamp when the session was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the session was last updated.
    pub updated_at: DateTime<Utc>,
}
