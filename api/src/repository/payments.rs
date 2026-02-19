//! Payment-focused repository operations.
//!
//! This module centralizes SQL queries used by payment management flows,
//! including scoped CRUD queries constrained to the authenticated user.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::payment::{Payment, PayoutType};

/// Input payload used by payment create and update repository operations.
#[derive(Debug, Clone)]
pub struct PaymentWriteInput {
    /// Company this payment belongs to.
    pub company_id: Uuid,
    /// Total payment amount.
    pub total: Decimal,
    /// Method by which the payment is received.
    pub payout_type: PayoutType,
    /// Expected payout date.
    pub expected_payout_date: Option<NaiveDate>,
    /// Expected transfer date for transfer-capable payout types.
    pub expected_transfer_date: Option<NaiveDate>,
    /// Whether transfer has been initiated.
    pub transfer_initiated: bool,
    /// Whether payment has been received.
    pub payment_received: bool,
    /// Whether transfer has been received.
    pub transfer_received: bool,
    /// Whether tax withholdings are covered.
    pub tax_withholdings_covered: bool,
}

/// Repository methods for payment-related persistence.
pub struct PaymentsRepo;

impl PaymentsRepo {
    /// Lists all payments owned by a specific user ordered by last update.
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
    ) -> Result<Vec<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            SELECT id, user_id, company_id, total, payout_type, expected_payout_date, expected_transfer_date,
                   transfer_initiated, payment_received, transfer_received, tax_withholdings_covered, created_at, updated_at
            FROM payments
            WHERE user_id = $1
            ORDER BY updated_at DESC, id DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Finds a single payment by ID scoped to the owner user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `payment_id` - Payment identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_by_id_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        payment_id: Uuid,
    ) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            SELECT id, user_id, company_id, total, payout_type, expected_payout_date, expected_transfer_date,
                   transfer_initiated, payment_received, transfer_received, tax_withholdings_covered, created_at, updated_at
            FROM payments
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(payment_id)
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

    /// Creates a payment for a user and returns the inserted record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `input` - Payment write fields to persist
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    pub async fn create_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        input: &PaymentWriteInput,
    ) -> Result<Payment, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            INSERT INTO payments (
                user_id,
                company_id,
                total,
                payout_type,
                expected_payout_date,
                expected_transfer_date,
                transfer_initiated,
                payment_received,
                transfer_received,
                tax_withholdings_covered
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, company_id, total, payout_type, expected_payout_date, expected_transfer_date,
                      transfer_initiated, payment_received, transfer_received, tax_withholdings_covered, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(input.company_id)
        .bind(input.total)
        .bind(input.payout_type)
        .bind(input.expected_payout_date)
        .bind(input.expected_transfer_date)
        .bind(input.transfer_initiated)
        .bind(input.payment_received)
        .bind(input.transfer_received)
        .bind(input.tax_withholdings_covered)
        .fetch_one(pool)
        .await
    }

    /// Updates a payment for a user and returns the updated record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `payment_id` - Payment identifier
    /// - `input` - Payment write fields to persist
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the update fails.
    pub async fn update_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        payment_id: Uuid,
        input: &PaymentWriteInput,
    ) -> Result<Option<Payment>, sqlx::Error> {
        sqlx::query_as::<_, Payment>(
            r#"
            UPDATE payments
            SET company_id = $1,
                total = $2,
                payout_type = $3,
                expected_payout_date = $4,
                expected_transfer_date = $5,
                transfer_initiated = $6,
                payment_received = $7,
                transfer_received = $8,
                tax_withholdings_covered = $9,
                updated_at = NOW()
            WHERE id = $10 AND user_id = $11
            RETURNING id, user_id, company_id, total, payout_type, expected_payout_date, expected_transfer_date,
                      transfer_initiated, payment_received, transfer_received, tax_withholdings_covered, created_at, updated_at
            "#,
        )
        .bind(input.company_id)
        .bind(input.total)
        .bind(input.payout_type)
        .bind(input.expected_payout_date)
        .bind(input.expected_transfer_date)
        .bind(input.transfer_initiated)
        .bind(input.payment_received)
        .bind(input.transfer_received)
        .bind(input.tax_withholdings_covered)
        .bind(payment_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Deletes a payment scoped to a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `payment_id` - Payment identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the delete fails.
    pub async fn delete_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        payment_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            DELETE FROM payments
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(payment_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
