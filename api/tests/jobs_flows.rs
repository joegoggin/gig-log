//! Integration tests for job routes.
//!
//! These tests cover authenticated job CRUD flows, user-scoped access
//! controls, company-ownership checks, and payment-type validation behavior
//! with real database persistence.

mod support;

use std::sync::{Mutex, MutexGuard, OnceLock};

use actix_web::{App, http::StatusCode, test, web};
use serde_json::json;
use sqlx::{Pool, Postgres};
use support::{app_state_with_mock_email, test_pool, unique_email};
use uuid::Uuid;

use api::auth::jwt::create_access_token;
use api::core::config::configure_routes;

fn test_guard() -> MutexGuard<'static, ()> {
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn insert_user(pool: &Pool<Postgres>, email: &str) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO users (first_name, last_name, email, hashed_password, email_confirmed)
        VALUES ('Test', 'User', $1, 'hashed-password', true)
        RETURNING id
        "#,
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("test user should be inserted")
}

async fn insert_company(pool: &Pool<Postgres>, user_id: Uuid, name: &str) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO companies (user_id, name, requires_tax_withholdings, tax_withholding_rate)
        VALUES ($1, $2, false, NULL)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(name)
    .fetch_one(pool)
    .await
    .expect("test company should be inserted")
}

fn access_cookie(token: &str) -> String {
    format!("access_token={token}")
}

#[actix_web::test]
// Verifies the full authenticated CRUD lifecycle for a job.
async fn jobs_crud_flow_succeeds_for_owner() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool.clone());
    let jwt_secret = state.env.jwt_secret.clone();
    let access_expiry = state.env.jwt_access_token_expiry_seconds;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("jobs-owner");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Acme Studio").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/jobs")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "company_id": company_id,
            "title": "Website Maintenance",
            "payment_type": "hourly",
            "number_of_payouts": null,
            "payout_amount": null,
            "hourly_rate": "55.50"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let job_id = Uuid::parse_str(
        create_body
            .get("job")
            .and_then(|job| job.get("id"))
            .and_then(|value| value.as_str())
            .expect("create response should include job id"),
    )
    .expect("job id should be a UUID");
    assert_eq!(
        create_body
            .get("job")
            .and_then(|job| job.get("payment_type"))
            .and_then(|value| value.as_str()),
        Some("hourly")
    );

    let list_request = test::TestRequest::get()
        .uri("/jobs")
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let list_response = test::call_service(&app, list_request).await;
    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body: serde_json::Value = test::read_body_json(list_response).await;
    let jobs = list_body
        .get("jobs")
        .and_then(|value| value.as_array())
        .expect("jobs list should be an array");
    assert_eq!(jobs.len(), 1);

    let get_request = test::TestRequest::get()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let get_response = test::call_service(&app, get_request).await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let update_request = test::TestRequest::put()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "company_id": company_id,
            "title": "Website Maintenance Retainer",
            "payment_type": "payouts",
            "number_of_payouts": 2,
            "payout_amount": "350.00",
            "hourly_rate": null
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_response).await;
    assert_eq!(
        update_body
            .get("job")
            .and_then(|job| job.get("title"))
            .and_then(|value| value.as_str()),
        Some("Website Maintenance Retainer")
    );
    assert_eq!(
        update_body
            .get("job")
            .and_then(|job| job.get("payment_type"))
            .and_then(|value| value.as_str()),
        Some("payouts")
    );
    assert_eq!(
        update_body
            .get("job")
            .and_then(|job| job.get("number_of_payouts"))
            .and_then(|value| value.as_i64()),
        Some(2)
    );
    assert!(
        update_body
            .get("job")
            .and_then(|job| job.get("hourly_rate"))
            .map(serde_json::Value::is_null)
            .unwrap_or(false)
    );

    let delete_request = test::TestRequest::delete()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let delete_response = test::call_service(&app, delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_deleted_request = test::TestRequest::get()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let get_deleted_response = test::call_service(&app, get_deleted_request).await;
    assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies users cannot access jobs they do not own and cannot use unowned companies.
async fn job_routes_enforce_scoping_and_company_ownership() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool.clone());
    let jwt_secret = state.env.jwt_secret.clone();
    let access_expiry = state.env.jwt_access_token_expiry_seconds;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let owner_email = unique_email("jobs-owner-scope");
    let other_email = unique_email("jobs-other-scope");
    let owner_id = insert_user(&pool, &owner_email).await;
    let other_id = insert_user(&pool, &other_email).await;
    let owner_company_id = insert_company(&pool, owner_id, "Owner Company").await;
    let other_company_id = insert_company(&pool, other_id, "Other Company").await;
    let owner_token = create_access_token(owner_id, &owner_email, &jwt_secret, access_expiry)
        .expect("owner access token should be created");
    let other_token = create_access_token(other_id, &other_email, &jwt_secret, access_expiry)
        .expect("other access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/jobs")
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "company_id": owner_company_id,
            "title": "Scoped Job",
            "payment_type": "hourly",
            "number_of_payouts": null,
            "payout_amount": null,
            "hourly_rate": "42.00"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let job_id = create_body
        .get("job")
        .and_then(|job| job.get("id"))
        .and_then(|value| value.as_str())
        .expect("created job id should exist");

    let get_request = test::TestRequest::get()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .to_request();
    let get_response = test::call_service(&app, get_request).await;
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    let update_request = test::TestRequest::put()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .set_json(json!({
            "company_id": other_company_id,
            "title": "Intrusion Update",
            "payment_type": "hourly",
            "number_of_payouts": null,
            "payout_amount": null,
            "hourly_rate": "50.00"
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);

    let delete_request = test::TestRequest::delete()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .to_request();
    let delete_response = test::call_service(&app, delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::NOT_FOUND);

    let create_unowned_company_request = test::TestRequest::post()
        .uri("/jobs")
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "company_id": other_company_id,
            "title": "Should Fail",
            "payment_type": "hourly",
            "number_of_payouts": null,
            "payout_amount": null,
            "hourly_rate": "60.00"
        }))
        .to_request();
    let create_unowned_company_response =
        test::call_service(&app, create_unowned_company_request).await;
    assert_eq!(
        create_unowned_company_response.status(),
        StatusCode::NOT_FOUND
    );

    let owner_update_unowned_company_request = test::TestRequest::put()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "company_id": other_company_id,
            "title": "Should Also Fail",
            "payment_type": "hourly",
            "number_of_payouts": null,
            "payout_amount": null,
            "hourly_rate": "61.00"
        }))
        .to_request();
    let owner_update_unowned_company_response =
        test::call_service(&app, owner_update_unowned_company_request).await;
    assert_eq!(
        owner_update_unowned_company_response.status(),
        StatusCode::NOT_FOUND
    );

    let owner_get_request = test::TestRequest::get()
        .uri(&format!("/jobs/{job_id}"))
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .to_request();
    let owner_get_response = test::call_service(&app, owner_get_request).await;
    assert_eq!(owner_get_response.status(), StatusCode::OK);

    let owner_get_body: serde_json::Value = test::read_body_json(owner_get_response).await;
    let owner_company_id_string = owner_company_id.to_string();
    assert_eq!(
        owner_get_body
            .get("job")
            .and_then(|job| job.get("company_id"))
            .and_then(|value| value.as_str()),
        Some(owner_company_id_string.as_str())
    );
}

#[actix_web::test]
// Verifies invalid job payloads return standardized validation-error shape.
async fn create_job_with_invalid_payload_returns_validation_errors() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool.clone());
    let jwt_secret = state.env.jwt_secret.clone();
    let access_expiry = state.env.jwt_access_token_expiry_seconds;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("jobs-validation");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Validation Company").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let request = test::TestRequest::post()
        .uri("/jobs")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "company_id": company_id,
            "title": "Invalid Payout Job",
            "payment_type": "payouts",
            "number_of_payouts": null,
            "payout_amount": null,
            "hourly_rate": "35.00"
        }))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(response).await;
    let errors = body
        .get("errors")
        .and_then(|value| value.as_array())
        .expect("validation errors should be an array");

    assert!(
        errors
            .iter()
            .any(|error| error.get("field") == Some(&json!("payout_fields_required")))
    );
    assert!(!errors.is_empty());
}
