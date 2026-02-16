//! Company-focused repository operations.
//!
//! This module centralizes SQL queries used by company management flows,
//! including scoped CRUD queries constrained to the authenticated user.

use rust_decimal::Decimal;
use sqlx::{FromRow, Pool, Postgres};
use uuid::Uuid;

use crate::models::company::Company;
use crate::models::payment::PayoutType;

/// Job list item row used by company-detail pagination queries.
#[derive(Debug, FromRow)]
pub struct CompanyJobSummaryRow {
    /// Unique identifier for the job.
    pub id: Uuid,
    /// Job title.
    pub title: String,
}

/// Payment list item row used by company-detail pagination queries.
#[derive(Debug, FromRow)]
pub struct CompanyPaymentSummaryRow {
    /// Unique identifier for the payment.
    pub id: Uuid,
    /// Total payment amount.
    pub total: Decimal,
    /// Payout method.
    pub payout_type: PayoutType,
    /// Whether payment was received.
    pub payment_received: bool,
    /// Whether transfer was received.
    pub transfer_received: bool,
}

/// Repository methods for company-related persistence.
pub struct CompaniesRepo;

impl CompaniesRepo {
    const COMPANY_DETAIL_PAGE_SIZE: usize = 5;

    /// Lists all companies owned by a specific user ordered by creation date.
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
    ) -> Result<Vec<Company>, sqlx::Error> {
        sqlx::query_as::<_, Company>(
            r#"
            SELECT id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at
            FROM companies
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Finds a single company by ID scoped to the owner user.
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
    pub async fn find_by_id_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Option<Company>, sqlx::Error> {
        sqlx::query_as::<_, Company>(
            r#"
            SELECT id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at
            FROM companies
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Creates a company for a user and returns the inserted record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `name` - Company display name
    /// - `requires_tax_withholdings` - Whether the company requires tax withholding handling
    /// - `tax_withholding_rate` - Optional withholding rate percentage
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        name: &str,
        requires_tax_withholdings: bool,
        tax_withholding_rate: Option<Decimal>,
    ) -> Result<Company, sqlx::Error> {
        sqlx::query_as::<_, Company>(
            r#"
            INSERT INTO companies (user_id, name, requires_tax_withholdings, tax_withholding_rate)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(requires_tax_withholdings)
        .bind(tax_withholding_rate)
        .fetch_one(pool)
        .await
    }

    /// Updates a company for a user and returns the updated record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `company_id` - Company identifier
    /// - `name` - Company display name
    /// - `requires_tax_withholdings` - Whether the company requires tax withholding handling
    /// - `tax_withholding_rate` - Optional withholding rate percentage
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn update_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
        name: &str,
        requires_tax_withholdings: bool,
        tax_withholding_rate: Option<Decimal>,
    ) -> Result<Option<Company>, sqlx::Error> {
        sqlx::query_as::<_, Company>(
            r#"
            UPDATE companies
            SET name = $1,
                requires_tax_withholdings = $2,
                tax_withholding_rate = $3,
                updated_at = NOW()
            WHERE id = $4 AND user_id = $5
            RETURNING id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at
            "#,
        )
        .bind(name)
        .bind(requires_tax_withholdings)
        .bind(tax_withholding_rate)
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Deletes a company scoped to a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `company_id` - Company identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the delete fails.
    pub async fn delete_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM companies
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Returns paginated jobs for a company scoped to a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `company_id` - Company identifier
    /// - `page` - 1-indexed page number
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn list_company_jobs_page_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
        page: usize,
    ) -> Result<(bool, Vec<CompanyJobSummaryRow>), sqlx::Error> {
        let offset = (page.saturating_sub(1) * Self::COMPANY_DETAIL_PAGE_SIZE) as i64;
        let limit = (Self::COMPANY_DETAIL_PAGE_SIZE + 1) as i64;

        let mut rows = sqlx::query_as::<_, CompanyJobSummaryRow>(
            r#"
            SELECT j.id, j.title
            FROM jobs j
            WHERE j.company_id = $1
              AND j.user_id = $2
            ORDER BY j.updated_at DESC, j.id DESC
            OFFSET $3
            LIMIT $4
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let has_more = rows.len() > Self::COMPANY_DETAIL_PAGE_SIZE;

        if has_more {
            rows.truncate(Self::COMPANY_DETAIL_PAGE_SIZE);
        }

        Ok((has_more, rows))
    }

    /// Returns paginated payments for a company scoped to a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `company_id` - Company identifier
    /// - `page` - 1-indexed page number
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn list_company_payments_page_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
        page: usize,
    ) -> Result<(bool, Vec<CompanyPaymentSummaryRow>), sqlx::Error> {
        let offset = (page.saturating_sub(1) * Self::COMPANY_DETAIL_PAGE_SIZE) as i64;
        let limit = (Self::COMPANY_DETAIL_PAGE_SIZE + 1) as i64;

        let mut rows = sqlx::query_as::<_, CompanyPaymentSummaryRow>(
            r#"
            SELECT p.id, p.total, p.payout_type, p.payment_received, p.transfer_received
            FROM payments p
            WHERE p.company_id = $1
              AND p.user_id = $2
            ORDER BY p.updated_at DESC, p.id DESC
            OFFSET $3
            LIMIT $4
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .bind(offset)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        let has_more = rows.len() > Self::COMPANY_DETAIL_PAGE_SIZE;

        if has_more {
            rows.truncate(Self::COMPANY_DETAIL_PAGE_SIZE);
        }

        Ok((has_more, rows))
    }

    /// Returns the total payment amount for a company scoped to a user.
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
    pub async fn payment_total_for_company_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<Decimal, sqlx::Error> {
        let total = sqlx::query_scalar::<_, Option<Decimal>>(
            r#"
            SELECT SUM(p.total)
            FROM payments p
            WHERE p.company_id = $1
              AND p.user_id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?
        .unwrap_or(Decimal::ZERO);

        Ok(total)
    }

    /// Returns total worked duration for a company scoped to a user.
    ///
    /// The returned string is formatted as `Xh Ym`.
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
    pub async fn total_hours_for_company_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        company_id: Uuid,
    ) -> Result<String, sqlx::Error> {
        let total_seconds = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COALESCE(
                SUM(
                    GREATEST(
                        CASE
                            WHEN ws.start_time IS NOT NULL AND ws.end_time IS NOT NULL THEN
                                EXTRACT(EPOCH FROM (ws.end_time - ws.start_time))::BIGINT
                                - ws.accumulated_paused_duration
                            ELSE 0
                        END,
                        0
                    )
                ),
                0
            )::BIGINT
            FROM work_sessions ws
            JOIN jobs j ON j.id = ws.job_id
            WHERE j.company_id = $1
              AND j.user_id = $2
              AND ws.user_id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let total_minutes = total_seconds / 60;
        let hours = total_minutes / 60;
        let minutes = total_minutes % 60;

        Ok(format!("{hours}h {minutes}m"))
    }
}
