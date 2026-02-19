//! Work-session-focused repository operations.
//!
//! This module centralizes SQL queries used by work session management flows,
//! including scoped CRUD queries constrained to the authenticated user.

use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::work_session::WorkSession;

/// Input payload used by work session create and update repository operations.
#[derive(Debug, Clone)]
pub struct WorkSessionWriteInput {
    /// Job this work session belongs to.
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
}

/// Repository methods for work-session-related persistence.
pub struct WorkSessionsRepo;

impl WorkSessionsRepo {
    /// Lists all work sessions owned by a specific user ordered by last update.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn list_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> Result<Vec<WorkSession>, sqlx::Error> {
        sqlx::query_as::<_, WorkSession>(
            r#"
            SELECT id, user_id, job_id, start_time, end_time, is_running,
                   accumulated_paused_duration, paused_at, time_reported, created_at, updated_at
            FROM work_sessions
            WHERE user_id = $1
            ORDER BY updated_at DESC, id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Lists all work sessions for a specific job owned by a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `job_id` - Job identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn list_for_job_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        job_id: Uuid,
    ) -> Result<Vec<WorkSession>, sqlx::Error> {
        sqlx::query_as::<_, WorkSession>(
            r#"
            SELECT id, user_id, job_id, start_time, end_time, is_running,
                   accumulated_paused_duration, paused_at, time_reported, created_at, updated_at
            FROM work_sessions
            WHERE user_id = $1 AND job_id = $2
            ORDER BY updated_at DESC, id DESC
            "#,
        )
        .bind(user_id)
        .bind(job_id)
        .fetch_all(pool)
        .await
    }

    /// Finds a single work session by ID scoped to the owner user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `work_session_id` - Work session identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_by_id_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        work_session_id: Uuid,
    ) -> Result<Option<WorkSession>, sqlx::Error> {
        sqlx::query_as::<_, WorkSession>(
            r#"
            SELECT id, user_id, job_id, start_time, end_time, is_running,
                   accumulated_paused_duration, paused_at, time_reported, created_at, updated_at
            FROM work_sessions
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(work_session_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Checks whether a job exists and is owned by the given user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `job_id` - Job identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn job_exists_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        job_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM jobs
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(job_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Creates a work session for a user and returns the inserted record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `input` - Work session write fields to persist
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        input: &WorkSessionWriteInput,
    ) -> Result<WorkSession, sqlx::Error> {
        sqlx::query_as::<_, WorkSession>(
            r#"
            INSERT INTO work_sessions (
                user_id,
                job_id,
                start_time,
                end_time,
                is_running,
                accumulated_paused_duration,
                paused_at,
                time_reported
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, job_id, start_time, end_time, is_running,
                      accumulated_paused_duration, paused_at, time_reported, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(input.job_id)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.is_running)
        .bind(input.accumulated_paused_duration)
        .bind(input.paused_at)
        .bind(input.time_reported)
        .fetch_one(pool)
        .await
    }

    /// Updates a work session for a user and returns the updated record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `work_session_id` - Work session identifier
    /// - `input` - Work session write fields to persist
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn update_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        work_session_id: Uuid,
        input: &WorkSessionWriteInput,
    ) -> Result<Option<WorkSession>, sqlx::Error> {
        sqlx::query_as::<_, WorkSession>(
            r#"
            UPDATE work_sessions
            SET job_id = $1,
                start_time = $2,
                end_time = $3,
                is_running = $4,
                accumulated_paused_duration = $5,
                paused_at = $6,
                time_reported = $7,
                updated_at = NOW()
            WHERE id = $8 AND user_id = $9
            RETURNING id, user_id, job_id, start_time, end_time, is_running,
                      accumulated_paused_duration, paused_at, time_reported, created_at, updated_at
            "#,
        )
        .bind(input.job_id)
        .bind(input.start_time)
        .bind(input.end_time)
        .bind(input.is_running)
        .bind(input.accumulated_paused_duration)
        .bind(input.paused_at)
        .bind(input.time_reported)
        .bind(work_session_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Deletes a work session scoped to a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `work_session_id` - Work session identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the delete fails.
    pub async fn delete_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        work_session_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM work_sessions
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(work_session_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
