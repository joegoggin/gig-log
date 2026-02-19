//! HTTP handler functions for payment endpoints.
//!
//! This module contains handlers for listing, viewing, creating, updating,
//! and deleting payments for authenticated users.

use actix_web::{HttpResponse, delete, get, post, put, web};
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::core::app_state::AppState;
use crate::core::error::{ApiError, ApiResult};
use crate::extractors::ValidatedJson;
use crate::repository::payments::{PaymentWriteInput, PaymentsRepo};

use super::payloads::{
    CreatePaymentRequest, DeletePaymentResponse, PaymentResponse, PaymentsListResponse,
    UpdatePaymentRequest,
};

/// Lists all payments owned by the authenticated user.
///
/// # Route
///
/// `GET /payments`
///
/// # Response Body ([`PaymentsListResponse`])
///
/// - `payments` - All payments owned by the authenticated user
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `DatabaseError` - If payment retrieval fails
#[get("/payments")]
pub async fn list_payments(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> ApiResult<HttpResponse> {
    let payments = PaymentsRepo::list_for_user(&state.pool, auth_user.user_id).await?;

    Ok(HttpResponse::Ok().json(PaymentsListResponse { payments }))
}

/// Retrieves a payment by identifier for the authenticated user.
///
/// # Route
///
/// `GET /payments/{payment_id}`
///
/// # Response Body ([`PaymentResponse`])
///
/// - `payment` - The requested payment resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no payment exists with that ID for the authenticated user
/// - `DatabaseError` - If payment retrieval fails
#[get("/payments/{payment_id}")]
pub async fn get_payment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    payment_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let payment = PaymentsRepo::find_by_id_for_user(&state.pool, auth_user.user_id, *payment_id)
        .await?
        .ok_or(ApiError::NotFound("Payment not found".to_string()))?;

    Ok(HttpResponse::Ok().json(PaymentResponse { payment }))
}

/// Creates a payment for the authenticated user.
///
/// # Route
///
/// `POST /payments`
///
/// # Request Body ([`CreatePaymentRequest`])
///
/// - `company_id` - Company identifier the payment belongs to
/// - `total` - Total payment amount (must be greater than 0)
/// - `payout_type` - Method by which the payment is received
/// - `expected_payout_date` - Optional expected payout date
/// - `expected_transfer_date` - Optional expected transfer date
/// - `transfer_initiated` - Whether transfer has been initiated
/// - `payment_received` - Whether payment has been received
/// - `transfer_received` - Whether transferred funds have been received
/// - `tax_withholdings_covered` - Whether tax withholdings are covered
///
/// # Response Body ([`PaymentResponse`])
///
/// - `payment` - The newly created payment resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request body validation fails
/// - `NotFound` - If the target company does not exist for the authenticated user
/// - `DatabaseError` - If payment creation fails
#[post("/payments")]
pub async fn create_payment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    body: ValidatedJson<CreatePaymentRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let company_exists =
        PaymentsRepo::company_exists_for_user(&state.pool, auth_user.user_id, body.company_id)
            .await?;

    if !company_exists {
        return Err(ApiError::NotFound("Company not found".to_string()));
    }

    let input = PaymentWriteInput {
        company_id: body.company_id,
        total: body.total,
        payout_type: body.payout_type,
        expected_payout_date: body.expected_payout_date,
        expected_transfer_date: body.expected_transfer_date,
        transfer_initiated: body.transfer_initiated,
        payment_received: body.payment_received,
        transfer_received: body.transfer_received,
        tax_withholdings_covered: body.tax_withholdings_covered,
    };

    let payment = PaymentsRepo::create_for_user(&state.pool, auth_user.user_id, &input).await?;

    Ok(HttpResponse::Created().json(PaymentResponse { payment }))
}

/// Updates a payment for the authenticated user.
///
/// # Route
///
/// `PUT /payments/{payment_id}`
///
/// # Request Body ([`UpdatePaymentRequest`])
///
/// - `company_id` - Company identifier the payment belongs to
/// - `total` - Total payment amount (must be greater than 0)
/// - `payout_type` - Method by which the payment is received
/// - `expected_payout_date` - Optional expected payout date
/// - `expected_transfer_date` - Optional expected transfer date
/// - `transfer_initiated` - Whether transfer has been initiated
/// - `payment_received` - Whether payment has been received
/// - `transfer_received` - Whether transferred funds have been received
/// - `tax_withholdings_covered` - Whether tax withholdings are covered
///
/// # Response Body ([`PaymentResponse`])
///
/// - `payment` - The updated payment resource
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request body validation fails
/// - `NotFound` - If no payment exists with that ID for the authenticated user
/// - `NotFound` - If the target company does not exist for the authenticated user
/// - `DatabaseError` - If payment update fails
#[put("/payments/{payment_id}")]
pub async fn update_payment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    payment_id: web::Path<Uuid>,
    body: ValidatedJson<UpdatePaymentRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let company_exists =
        PaymentsRepo::company_exists_for_user(&state.pool, auth_user.user_id, body.company_id)
            .await?;

    if !company_exists {
        return Err(ApiError::NotFound("Company not found".to_string()));
    }

    let input = PaymentWriteInput {
        company_id: body.company_id,
        total: body.total,
        payout_type: body.payout_type,
        expected_payout_date: body.expected_payout_date,
        expected_transfer_date: body.expected_transfer_date,
        transfer_initiated: body.transfer_initiated,
        payment_received: body.payment_received,
        transfer_received: body.transfer_received,
        tax_withholdings_covered: body.tax_withholdings_covered,
    };

    let payment =
        PaymentsRepo::update_for_user(&state.pool, auth_user.user_id, *payment_id, &input)
            .await?
            .ok_or(ApiError::NotFound("Payment not found".to_string()))?;

    Ok(HttpResponse::Ok().json(PaymentResponse { payment }))
}

/// Deletes a payment for the authenticated user.
///
/// # Route
///
/// `DELETE /payments/{payment_id}`
///
/// # Response Body ([`DeletePaymentResponse`])
///
/// - `message` - Deletion status message
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no payment exists with that ID for the authenticated user
/// - `DatabaseError` - If payment deletion fails
#[delete("/payments/{payment_id}")]
pub async fn delete_payment(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    payment_id: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let deleted =
        PaymentsRepo::delete_for_user(&state.pool, auth_user.user_id, *payment_id).await?;

    if !deleted {
        return Err(ApiError::NotFound("Payment not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(DeletePaymentResponse {
        message: "Payment deleted successfully.".to_string(),
    }))
}

#[cfg(test)]
mod tests {
    //! Handler-level tests for payment routes.
    //!
    //! Covered behavior:
    //! - Auth guard rejects unauthenticated payment requests.
    //! - Request validation rejects invalid payment payloads.
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

    use super::{create_payment, get_payment, list_payments};

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
    async fn list_payments_without_access_cookie_returns_unauthorized() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .service(list_payments),
        )
        .await;

        let request = test::TestRequest::get().uri("/payments").to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies create requests reject invalid payout/date/status payload combinations.
    async fn create_payment_with_invalid_status_configuration_returns_bad_request() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "payment-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(create_payment),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/payments")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .set_json(serde_json::json!({
                "company_id": Uuid::new_v4(),
                "total": "0.00",
                "payout_type": "cash",
                "expected_payout_date": null,
                "expected_transfer_date": null,
                "transfer_initiated": false,
                "payment_received": false,
                "transfer_received": false,
                "tax_withholdings_covered": false
            }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    // Verifies malformed UUID path segments fail route matching.
    async fn get_payment_with_invalid_uuid_path_returns_not_found() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "payment-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(get_payment),
        )
        .await;

        let request = test::TestRequest::get()
            .uri("/payments/not-a-uuid")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
