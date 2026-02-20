//! HTTP handler functions for job endpoints.
//!
//! This module contains handlers for listing, viewing, creating, updating,
//! and deleting jobs for authenticated users.

use actix_web::{HttpResponse, delete, get, post, put, web};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::core::app_state::AppState;
use crate::core::error::{ApiError, ApiResult};
use crate::extractors::ValidatedJson;
use crate::models::job::PaymentType;
use crate::repository::jobs::{JobWriteInput, JobsRepo};

use super::payloads::{
    CreateJobRequest, DeleteJobResponse, JobResponse, JobsListResponse, UpdateJobRequest,
};

/// Normalizes payment-specific fields to keep persisted records canonical.
fn normalize_payment_fields(
    payment_type: PaymentType,
    number_of_payouts: Option<i32>,
    payout_amount: Option<Decimal>,
    hourly_rate: Option<Decimal>,
) -> (Option<i32>, Option<Decimal>, Option<Decimal>) {
    match payment_type {
        PaymentType::Hourly => (None, None, hourly_rate),
        PaymentType::Payouts => (number_of_payouts, payout_amount, None),
    }
}

/// Lists all jobs owned by the authenticated user.
///
/// # Route
///
/// `GET /jobs`
///
/// # Response Body ([`JobsListResponse`])
///
/// - `jobs` - All jobs owned by the authenticated user
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `DatabaseError` - If job retrieval fails
#[get("/jobs")]
pub async fn list_jobs(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> ApiResult<HttpResponse> {
    let jobs = JobsRepo::list_for_user(&state.pool, auth_user.user_id).await?;

    Ok(HttpResponse::Ok().json(JobsListResponse { jobs }))
}

/// Retrieves a job by identifier for the authenticated user.
///
/// # Route
///
/// `GET /jobs/{job_id}`
///
/// # Response Body ([`JobResponse`])
///
/// - `job` - The requested job resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no job exists with that ID for the authenticated user
/// - `DatabaseError` - If job retrieval fails
#[get("/jobs/{job_id}")]
pub async fn get_job(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    job_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let job = JobsRepo::find_by_id_for_user(&state.pool, auth_user.user_id, *job_id)
        .await?
        .ok_or(ApiError::NotFound("Job not found".to_string()))?;

    Ok(HttpResponse::Ok().json(JobResponse { job }))
}

/// Creates a job for the authenticated user.
///
/// # Route
///
/// `POST /jobs`
///
/// # Request Body ([`CreateJobRequest`])
///
/// - `company_id` - Company identifier the job belongs to
/// - `title` - Job display title
/// - `payment_type` - Compensation model (`hourly` or `payouts`)
/// - `number_of_payouts` - Required for `payouts` jobs
/// - `payout_amount` - Required for `payouts` jobs
/// - `hourly_rate` - Required for `hourly` jobs
///
/// # Response Body ([`JobResponse`])
///
/// - `job` - The newly created job resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request body validation fails
/// - `NotFound` - If the target company does not exist for the authenticated user
/// - `DatabaseError` - If job creation fails
#[post("/jobs")]
pub async fn create_job(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    body: ValidatedJson<CreateJobRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let company_exists =
        JobsRepo::company_exists_for_user(&state.pool, auth_user.user_id, body.company_id).await?;

    if !company_exists {
        return Err(ApiError::NotFound("Company not found".to_string()));
    }

    let (number_of_payouts, payout_amount, hourly_rate) = normalize_payment_fields(
        body.payment_type,
        body.number_of_payouts,
        body.payout_amount,
        body.hourly_rate,
    );

    let input = JobWriteInput {
        company_id: body.company_id,
        title: body.title,
        payment_type: body.payment_type,
        number_of_payouts,
        payout_amount,
        hourly_rate,
    };

    let job = JobsRepo::create_for_user(&state.pool, auth_user.user_id, &input).await?;

    Ok(HttpResponse::Created().json(JobResponse { job }))
}

/// Updates a job for the authenticated user.
///
/// # Route
///
/// `PUT /jobs/{job_id}`
///
/// # Request Body ([`UpdateJobRequest`])
///
/// - `company_id` - Company identifier the job belongs to
/// - `title` - Job display title
/// - `payment_type` - Compensation model (`hourly` or `payouts`)
/// - `number_of_payouts` - Required for `payouts` jobs
/// - `payout_amount` - Required for `payouts` jobs
/// - `hourly_rate` - Required for `hourly` jobs
///
/// # Response Body ([`JobResponse`])
///
/// - `job` - The updated job resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request body validation fails
/// - `NotFound` - If no job exists with that ID for the authenticated user
/// - `NotFound` - If the target company does not exist for the authenticated user
/// - `DatabaseError` - If job update fails
#[put("/jobs/{job_id}")]
pub async fn update_job(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    job_id: web::Path<Uuid>,
    body: ValidatedJson<UpdateJobRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let company_exists =
        JobsRepo::company_exists_for_user(&state.pool, auth_user.user_id, body.company_id).await?;

    if !company_exists {
        return Err(ApiError::NotFound("Company not found".to_string()));
    }

    let (number_of_payouts, payout_amount, hourly_rate) = normalize_payment_fields(
        body.payment_type,
        body.number_of_payouts,
        body.payout_amount,
        body.hourly_rate,
    );

    let input = JobWriteInput {
        company_id: body.company_id,
        title: body.title,
        payment_type: body.payment_type,
        number_of_payouts,
        payout_amount,
        hourly_rate,
    };

    let job = JobsRepo::update_for_user(&state.pool, auth_user.user_id, *job_id, &input)
        .await?
        .ok_or(ApiError::NotFound("Job not found".to_string()))?;

    Ok(HttpResponse::Ok().json(JobResponse { job }))
}

/// Deletes a job for the authenticated user.
///
/// # Route
///
/// `DELETE /jobs/{job_id}`
///
/// # Response Body ([`DeleteJobResponse`])
///
/// - `message` - Deletion status message
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no job exists with that ID for the authenticated user
/// - `DatabaseError` - If job deletion fails
#[delete("/jobs/{job_id}")]
pub async fn delete_job(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    job_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let deleted = JobsRepo::delete_for_user(&state.pool, auth_user.user_id, *job_id).await?;

    if !deleted {
        return Err(ApiError::NotFound("Job not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(DeleteJobResponse {
        message: "Job deleted successfully.".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    //! Handler-level tests for job routes.
    //!
    //! Covered behavior:
    //! - Auth guard rejects unauthenticated job requests.
    //! - Request validation rejects invalid job payloads.
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

    use super::{create_job, get_job, list_jobs};

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

        async fn send_email_change_email(
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
            cookie_domain: Some("localhost".to_string()),
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
    async fn list_jobs_without_access_cookie_returns_unauthorized() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .service(list_jobs),
        )
        .await;

        let request = test::TestRequest::get().uri("/jobs").to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies create requests reject invalid payment-configuration payloads.
    async fn create_job_with_invalid_payment_configuration_returns_bad_request() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "job-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(create_job),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/jobs")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .set_json(serde_json::json!({
                "company_id": Uuid::new_v4(),
                "title": "Invalid Hourly Job",
                "payment_type": "hourly",
                "number_of_payouts": 3,
                "payout_amount": "15.00",
                "hourly_rate": null
            }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    // Verifies malformed UUID path segments fail route matching.
    async fn get_job_with_invalid_uuid_path_returns_not_found() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "job-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app =
            test::init_service(App::new().app_data(web::Data::new(state)).service(get_job)).await;

        let request = test::TestRequest::get()
            .uri("/jobs/not-a-uuid")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
