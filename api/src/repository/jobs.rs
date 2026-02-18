//! Job-focused repository operations.
//!
//! This module centralizes SQL queries used by job management flows,
//! including scoped CRUD queries constrained to the authenticated user.

use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::job::{Job, PaymentType};

/// Repository methods for job-related persistence.
pub struct JobsRepo;

impl JobsRepo {
    /// Lists all jobs owned by a specific user ordered by last update.
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
    ) -> Result<Vec<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            SELECT id, company_id, user_id, title, payment_type, number_of_payouts, payout_amount, hourly_rate, created_at, updated_at
            FROM jobs
            WHERE user_id = $1
            ORDER BY updated_at DESC, id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Finds a single job by ID scoped to the owner user.
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
    pub async fn find_by_id_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        job_id: Uuid,
    ) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            SELECT id, company_id, user_id, title, payment_type, number_of_payouts, payout_amount, hourly_rate, created_at, updated_at
            FROM jobs
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(job_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Checks whether a company exists and is owned by the given user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `company_id` - Company identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn company_exists_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::BIGINT
            FROM companies
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        Ok(count > 0)
    }

    /// Creates a job for a user and returns the inserted record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `company_id` - Company this job belongs to
    /// - `title` - Job display title
    /// - `payment_type` - Compensation model for the job
    /// - `number_of_payouts` - Optional payout count for fixed payout jobs
    /// - `payout_amount` - Optional payout amount for fixed payout jobs
    /// - `hourly_rate` - Optional hourly rate for hourly jobs
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
        title: &str,
        payment_type: PaymentType,
        number_of_payouts: Option<i32>,
        payout_amount: Option<Decimal>,
        hourly_rate: Option<Decimal>,
    ) -> Result<Job, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            INSERT INTO jobs (company_id, user_id, title, payment_type, number_of_payouts, payout_amount, hourly_rate)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, company_id, user_id, title, payment_type, number_of_payouts, payout_amount, hourly_rate, created_at, updated_at
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .bind(title)
        .bind(payment_type)
        .bind(number_of_payouts)
        .bind(payout_amount)
        .bind(hourly_rate)
        .fetch_one(pool)
        .await
    }

    /// Updates a job for a user and returns the updated record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `job_id` - Job identifier
    /// - `company_id` - Company this job belongs to
    /// - `title` - Job display title
    /// - `payment_type` - Compensation model for the job
    /// - `number_of_payouts` - Optional payout count for fixed payout jobs
    /// - `payout_amount` - Optional payout amount for fixed payout jobs
    /// - `hourly_rate` - Optional hourly rate for hourly jobs
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn update_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        job_id: Uuid,
        company_id: Uuid,
        title: &str,
        payment_type: PaymentType,
        number_of_payouts: Option<i32>,
        payout_amount: Option<Decimal>,
        hourly_rate: Option<Decimal>,
    ) -> Result<Option<Job>, sqlx::Error> {
        sqlx::query_as::<_, Job>(
            r#"
            UPDATE jobs
            SET company_id = $1,
                title = $2,
                payment_type = $3,
                number_of_payouts = $4,
                payout_amount = $5,
                hourly_rate = $6,
                updated_at = NOW()
            WHERE id = $7 AND user_id = $8
            RETURNING id, company_id, user_id, title, payment_type, number_of_payouts, payout_amount, hourly_rate, created_at, updated_at
            "#,
        )
        .bind(company_id)
        .bind(title)
        .bind(payment_type)
        .bind(number_of_payouts)
        .bind(payout_amount)
        .bind(hourly_rate)
        .bind(job_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Deletes a job scoped to a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `job_id` - Job identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the delete fails.
    pub async fn delete_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        job_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM jobs
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(job_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
