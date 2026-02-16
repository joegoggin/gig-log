//! Company-focused repository operations.
//!
//! This module centralizes SQL queries used by company management flows,
//! including scoped CRUD queries constrained to the authenticated user.

use rust_decimal::Decimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::models::company::Company;

/// Repository methods for company-related persistence.
pub struct CompaniesRepo;

impl CompaniesRepo {
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
}
