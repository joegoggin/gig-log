//! HTTP handler functions for company endpoints.
//!
//! This module contains handlers for listing, viewing, creating, updating,
//! and deleting companies for authenticated users.

use actix_web::{HttpResponse, delete, get, post, put, web};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::core::app_state::AppState;
use crate::core::error::{ApiError, ApiResult};
use crate::extractors::ValidatedJson;
use crate::repository::companies::CompaniesRepo;

use super::payloads::{
    CompaniesListResponse, CompanyDetailResponse, CompanyDetails, CompanyJobSummary,
    CompanyPaymentSummary, CompanyResponse, CreateCompanyRequest, DeleteCompanyResponse,
    UpdateCompanyRequest,
};

/// Query-string pagination options for the company detail route.
#[derive(Debug, Deserialize)]
pub struct GetCompanyQuery {
    /// Optional jobs page override (1-indexed).
    pub jobs_page: Option<usize>,
    /// Optional payments page override (1-indexed).
    pub payments_page: Option<usize>,
}

/// Lists all companies owned by the authenticated user.
///
/// # Route
///
/// `GET /companies`
///
/// # Response Body ([`CompaniesListResponse`])
///
/// - `companies` - All companies owned by the authenticated user
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `DatabaseError` - If company retrieval fails
#[get("/companies")]
pub async fn list_companies(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> ApiResult<HttpResponse> {
    let companies = CompaniesRepo::list_for_user(&state.pool, auth_user.user_id).await?;

    Ok(HttpResponse::Ok().json(CompaniesListResponse { companies }))
}

/// Retrieves a company by identifier for the authenticated user.
///
/// This endpoint mirrors the Phoenix company-detail behavior by returning
/// paginated jobs and payments plus aggregate company metrics.
///
/// # Route
///
/// `GET /companies/{company_id}`
///
/// # Query Parameters
///
/// - `jobs_page` - Optional jobs page to load (1-indexed). When provided,
///   jobs are loaded for this page and payments are loaded from page `1`.
/// - `payments_page` - Optional payments page to load (1-indexed). Used only
///   when `jobs_page` is not present. Payments are loaded for this page and
///   jobs are loaded from page `1`.
///
/// # Response Body ([`CompanyDetailResponse`])
///
/// - `company` - The requested company resource plus aggregate metrics
/// - `paginated_jobs` - Jobs for the selected jobs page
/// - `jobs_has_more` - Whether additional job pages exist
/// - `paginated_payments` - Payments for the selected payments page
/// - `payments_has_more` - Whether additional payment pages exist
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no company exists with that ID for the authenticated user
/// - `DatabaseError` - If company retrieval fails
#[get("/companies/{company_id}")]
pub async fn get_company(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    company_id: web::Path<Uuid>,
    query: web::Query<GetCompanyQuery>,
) -> ApiResult<HttpResponse> {
    let company = CompaniesRepo::find_by_id_for_user(&state.pool, auth_user.user_id, *company_id)
        .await?
        .ok_or(ApiError::NotFound("Company not found".to_string()))?;

    let jobs_page = query.jobs_page.unwrap_or(1).max(1);
    let payments_page = if query.jobs_page.is_some() {
        1
    } else {
        query.payments_page.unwrap_or(1).max(1)
    };

    let (jobs_has_more, paginated_jobs_rows) = CompaniesRepo::list_company_jobs_page_for_user(
        &state.pool,
        auth_user.user_id,
        *company_id,
        jobs_page,
    )
    .await?;
    let (payments_has_more, paginated_payments_rows) =
        CompaniesRepo::list_company_payments_page_for_user(
            &state.pool,
            auth_user.user_id,
            *company_id,
            payments_page,
        )
        .await?;
    let payment_total = CompaniesRepo::payment_total_for_company_for_user(
        &state.pool,
        auth_user.user_id,
        *company_id,
    )
    .await?;
    let hours = CompaniesRepo::total_hours_for_company_for_user(
        &state.pool,
        auth_user.user_id,
        *company_id,
    )
    .await?;

    let paginated_jobs = paginated_jobs_rows
        .into_iter()
        .map(|job| CompanyJobSummary {
            id: job.id,
            title: job.title,
        })
        .collect();
    let paginated_payments = paginated_payments_rows
        .into_iter()
        .map(|payment| CompanyPaymentSummary {
            id: payment.id,
            total: payment.total,
            payout_type: payment.payout_type,
            payment_received: payment.payment_received,
            transfer_received: payment.transfer_received,
        })
        .collect();

    Ok(HttpResponse::Ok().json(CompanyDetailResponse {
        company: CompanyDetails {
            id: company.id,
            user_id: company.user_id,
            name: company.name,
            requires_tax_withholdings: company.requires_tax_withholdings,
            tax_withholding_rate: company.tax_withholding_rate,
            created_at: company.created_at,
            updated_at: company.updated_at,
            payment_total,
            hours,
        },
        paginated_jobs,
        jobs_has_more,
        paginated_payments,
        payments_has_more,
    }))
}

/// Creates a company for the authenticated user.
///
/// # Route
///
/// `POST /companies`
///
/// # Request Body ([`CreateCompanyRequest`])
///
/// - `name` - Company display name
/// - `requires_tax_withholdings` - Whether tax withholdings should be tracked
/// - `tax_withholding_rate` - Optional tax withholding percentage
///
/// # Response Body ([`CompanyResponse`])
///
/// - `company` - The newly created company resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request body validation fails
/// - `DatabaseError` - If company creation fails
#[post("/companies")]
pub async fn create_company(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    body: ValidatedJson<CreateCompanyRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let tax_withholding_rate = if body.requires_tax_withholdings {
        body.tax_withholding_rate
    } else {
        None
    };

    let company = CompaniesRepo::create_for_user(
        &state.pool,
        auth_user.user_id,
        &body.name,
        body.requires_tax_withholdings,
        tax_withholding_rate,
    )
    .await?;

    Ok(HttpResponse::Created().json(CompanyResponse { company }))
}

/// Updates a company for the authenticated user.
///
/// # Route
///
/// `PUT /companies/{company_id}`
///
/// # Request Body ([`UpdateCompanyRequest`])
///
/// - `name` - Company display name
/// - `requires_tax_withholdings` - Whether tax withholdings should be tracked
/// - `tax_withholding_rate` - Optional tax withholding percentage
///
/// # Response Body ([`CompanyResponse`])
///
/// - `company` - The updated company resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request body validation fails
/// - `NotFound` - If no company exists with that ID for the authenticated user
/// - `DatabaseError` - If company update fails
#[put("/companies/{company_id}")]
pub async fn update_company(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    company_id: web::Path<Uuid>,
    body: ValidatedJson<UpdateCompanyRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let tax_withholding_rate = if body.requires_tax_withholdings {
        body.tax_withholding_rate
    } else {
        None
    };

    let company = CompaniesRepo::update_for_user(
        &state.pool,
        auth_user.user_id,
        *company_id,
        &body.name,
        body.requires_tax_withholdings,
        tax_withholding_rate,
    )
    .await?
    .ok_or(ApiError::NotFound("Company not found".to_string()))?;

    Ok(HttpResponse::Ok().json(CompanyResponse { company }))
}

/// Deletes a company for the authenticated user.
///
/// # Route
///
/// `DELETE /companies/{company_id}`
///
/// # Response Body ([`DeleteCompanyResponse`])
///
/// - `message` - Deletion status message
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no company exists with that ID for the authenticated user
/// - `DatabaseError` - If company deletion fails
#[delete("/companies/{company_id}")]
pub async fn delete_company(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    company_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let deleted =
        CompaniesRepo::delete_for_user(&state.pool, auth_user.user_id, *company_id).await?;

    if !deleted {
        return Err(ApiError::NotFound("Company not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(DeleteCompanyResponse {
        message: "Company deleted successfully.".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    //! Handler-level tests for company routes.
    //!
    //! Covered behavior:
    //! - Auth guard rejects unauthenticated company requests.
    //! - Request validation rejects invalid company payloads.
    //! - Invalid path parameters return a client error before handler execution.

    use std::sync::Arc;

    use actix_web::{App, http::StatusCode, test, web};
    use async_trait::async_trait;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use crate::auth::jwt::create_access_token;
    use crate::core::app_state::AppState;
    use crate::core::env::Env;
    use crate::core::error::ApiError;
    use crate::services::email::EmailSender;

    use super::{create_company, get_company, list_companies};

    #[derive(Debug, Default)]
    struct NoopEmailSender;

    #[async_trait]
    impl EmailSender for NoopEmailSender {
        async fn send_confirmation_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }

        async fn send_password_reset_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }
    }

    fn test_env() -> Env {
        Env {
            app_env: "test".to_string(),
            docker_compose_auto_start_enabled: false,
            auto_apply_migrations_enabled: false,
            database_url: "postgres://localhost/test".to_string(),
            cors_allowed_origin: "http://localhost:3000".to_string(),
            port: 0,
            jwt_secret: "handler-test-secret".to_string(),
            jwt_access_token_expiry_seconds: 900,
            jwt_refresh_token_expiry_seconds: 604_800,
            resend_api_key: "test-resend-key".to_string(),
            resend_from_email: "test@giglog.dev".to_string(),
            auth_code_expiry_seconds: 600,
            cookie_domain: "localhost".to_string(),
            cookie_secure: false,
            log_level: "info".to_string(),
            log_http_body_enabled: false,
            log_http_max_body_bytes: 16_384,
        }
    }

    fn test_state() -> AppState {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://postgres:postgres@localhost:5432/postgres")
            .expect("lazy pool should be created for handler tests");

        AppState::with_email_sender(pool, test_env(), Arc::new(NoopEmailSender))
    }

    #[actix_web::test]
    // Verifies unauthenticated list requests are rejected by the auth extractor.
    async fn list_companies_without_access_cookie_returns_unauthorized() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .service(list_companies),
        )
        .await;

        let request = test::TestRequest::get().uri("/companies").to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies create requests reject invalid tax-configuration payloads.
    async fn create_company_with_invalid_tax_configuration_returns_bad_request() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "company-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(create_company),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/companies")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .set_json(serde_json::json!({
                "name": "Acme",
                "requires_tax_withholdings": true,
                "tax_withholding_rate": null
            }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    // Verifies malformed UUID path segments fail route matching.
    async fn get_company_with_invalid_uuid_path_returns_not_found() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "company-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(get_company),
        )
        .await;

        let request = test::TestRequest::get()
            .uri("/companies/not-a-uuid")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
