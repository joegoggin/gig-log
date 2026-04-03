use gig_log_common::models::company::Company;
use sqlx::types::BigDecimal;
use sqlx::{Pool, Postgres, query, query_as};
use uuid::Uuid;

use crate::core::error::ApiResult;

pub struct CompanyRepo;

impl CompanyRepo {
    pub async fn insert_company(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        name: String,
        requires_tax_withholdings: bool,
        tax_withholding_rate: Option<BigDecimal>,
    ) -> ApiResult<Company> {
        let company = query_as!(
            Company,
            r#"
            INSERT INTO companies (user_id, name, requires_tax_withholdings, tax_withholding_rate)
            VALUES($1, $2, $3, $4)
            RETURNING id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at  
            "#,
            user_id,
            name,
            requires_tax_withholdings,
            tax_withholding_rate
        )
        .fetch_one(pool)
        .await?;

        Ok(company)
    }

    pub async fn find_companies_by_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> ApiResult<Vec<Company>> {
        let companies = query_as!(
            Company,
            r#"
            SELECT id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at
            FROM companies 
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(companies)
    }

    pub async fn find_company_by_id(
        pool: &Pool<Postgres>,
        id: Uuid,
        user_id: Uuid,
    ) -> ApiResult<Option<Company>> {
        let company = query_as!(
            Company,
            r#"
            SELECT id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at
            FROM companies
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(company)
    }

    pub async fn update_company(
        pool: &Pool<Postgres>,
        id: Uuid,
        user_id: Uuid,
        name: Option<String>,
        requires_tax_withholdings: Option<bool>,
        tax_withholding_rate: Option<BigDecimal>,
    ) -> ApiResult<Company> {
        let company = query_as!(
            Company,
            r#"
            UPDATE companies
            SET
                name = COALESCE($1, name),
                requires_tax_withholdings = COALESCE($2, requires_tax_withholdings),
                tax_withholding_rate = COALESCE($3, tax_withholding_rate),
                updated_at = NOW()
            WHERE id = $4 AND user_id = $5
            RETURNING id, user_id, name, requires_tax_withholdings, tax_withholding_rate, created_at, updated_at  
            "#,
            name,
            requires_tax_withholdings,
            tax_withholding_rate,
            id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(company)
    }

    pub async fn delete_company(pool: &Pool<Postgres>, id: Uuid, user_id: Uuid) -> ApiResult<()> {
        query!(
            r#"
            DELETE FROM companies
            WHERE id = $1 AND user_id = $2
            "#,
            id,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(())
    }
}
