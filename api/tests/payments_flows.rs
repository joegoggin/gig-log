//! Integration tests for payment routes.
//!
//! These tests cover authenticated payment CRUD flows, user-scoped access
//! controls, company-ownership checks, and payout/date/status validation
//! behavior with real database persistence.

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
// Verifies the full authenticated CRUD lifecycle for a payment.
async fn payments_crud_flow_succeeds_for_owner() {
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

    let email = unique_email("payments-owner");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Acme Studio").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/payments")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "company_id": company_id,
            "total": "250.00",
            "payout_type": "paypal",
            "expected_payout_date": "2026-02-01",
            "expected_transfer_date": "2026-02-03",
            "transfer_initiated": true,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": true
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let payment_id = Uuid::parse_str(
        create_body
            .get("payment")
            .and_then(|payment| payment.get("id"))
            .and_then(|value| value.as_str())
            .expect("create response should include payment id"),
    )
    .expect("payment id should be a UUID");

    let list_request = test::TestRequest::get()
        .uri("/payments")
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let list_response = test::call_service(&app, list_request).await;
    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body: serde_json::Value = test::read_body_json(list_response).await;
    let payments = list_body
        .get("payments")
        .and_then(|value| value.as_array())
        .expect("payments list should be an array");
    assert_eq!(payments.len(), 1);

    let get_request = test::TestRequest::get()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let get_response = test::call_service(&app, get_request).await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let update_request = test::TestRequest::put()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "company_id": company_id,
            "total": "300.00",
            "payout_type": "cash",
            "expected_payout_date": "2026-02-05",
            "expected_transfer_date": null,
            "transfer_initiated": false,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": false
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_response).await;
    assert_eq!(
        update_body
            .get("payment")
            .and_then(|payment| payment.get("total"))
            .and_then(|value| value.as_str()),
        Some("300.00")
    );
    assert_eq!(
        update_body
            .get("payment")
            .and_then(|payment| payment.get("payout_type"))
            .and_then(|value| value.as_str()),
        Some("cash")
    );
    assert!(
        update_body
            .get("payment")
            .and_then(|payment| payment.get("expected_transfer_date"))
            .map(serde_json::Value::is_null)
            .unwrap_or(false)
    );

    let delete_request = test::TestRequest::delete()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let delete_response = test::call_service(&app, delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_deleted_request = test::TestRequest::get()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let get_deleted_response = test::call_service(&app, get_deleted_request).await;
    assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies users cannot access payments they do not own and cannot use unowned companies.
async fn payment_routes_enforce_scoping_and_company_ownership() {
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

    let owner_email = unique_email("payments-owner-scope");
    let other_email = unique_email("payments-other-scope");
    let owner_id = insert_user(&pool, &owner_email).await;
    let other_id = insert_user(&pool, &other_email).await;
    let owner_company_id = insert_company(&pool, owner_id, "Owner Company").await;
    let other_company_id = insert_company(&pool, other_id, "Other Company").await;
    let owner_token = create_access_token(owner_id, &owner_email, &jwt_secret, access_expiry)
        .expect("owner access token should be created");
    let other_token = create_access_token(other_id, &other_email, &jwt_secret, access_expiry)
        .expect("other access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/payments")
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "company_id": owner_company_id,
            "total": "125.00",
            "payout_type": "venmo",
            "expected_payout_date": "2026-02-01",
            "expected_transfer_date": "2026-02-03",
            "transfer_initiated": false,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": false
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let payment_id = create_body
        .get("payment")
        .and_then(|payment| payment.get("id"))
        .and_then(|value| value.as_str())
        .expect("created payment id should exist");

    let get_request = test::TestRequest::get()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .to_request();
    let get_response = test::call_service(&app, get_request).await;
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    let update_request = test::TestRequest::put()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .set_json(json!({
            "company_id": other_company_id,
            "total": "150.00",
            "payout_type": "cash",
            "expected_payout_date": "2026-02-05",
            "expected_transfer_date": null,
            "transfer_initiated": false,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": false
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);

    let delete_request = test::TestRequest::delete()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .to_request();
    let delete_response = test::call_service(&app, delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::NOT_FOUND);

    let create_unowned_company_request = test::TestRequest::post()
        .uri("/payments")
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "company_id": other_company_id,
            "total": "199.00",
            "payout_type": "cash",
            "expected_payout_date": "2026-02-06",
            "expected_transfer_date": null,
            "transfer_initiated": false,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": false
        }))
        .to_request();
    let create_unowned_company_response =
        test::call_service(&app, create_unowned_company_request).await;
    assert_eq!(
        create_unowned_company_response.status(),
        StatusCode::NOT_FOUND
    );

    let owner_update_unowned_company_request = test::TestRequest::put()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "company_id": other_company_id,
            "total": "201.00",
            "payout_type": "cash",
            "expected_payout_date": "2026-02-07",
            "expected_transfer_date": null,
            "transfer_initiated": false,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": false
        }))
        .to_request();
    let owner_update_unowned_company_response =
        test::call_service(&app, owner_update_unowned_company_request).await;
    assert_eq!(
        owner_update_unowned_company_response.status(),
        StatusCode::NOT_FOUND
    );

    let owner_get_request = test::TestRequest::get()
        .uri(&format!("/payments/{payment_id}"))
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .to_request();
    let owner_get_response = test::call_service(&app, owner_get_request).await;
    assert_eq!(owner_get_response.status(), StatusCode::OK);

    let owner_get_body: serde_json::Value = test::read_body_json(owner_get_response).await;
    let owner_company_id_string = owner_company_id.to_string();
    assert_eq!(
        owner_get_body
            .get("payment")
            .and_then(|payment| payment.get("company_id"))
            .and_then(|value| value.as_str()),
        Some(owner_company_id_string.as_str())
    );
}

#[actix_web::test]
// Verifies invalid payment payloads return standardized validation-error shape.
async fn create_payment_with_invalid_payload_returns_validation_errors() {
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

    let email = unique_email("payments-validation");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Validation Company").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let request = test::TestRequest::post()
        .uri("/payments")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "company_id": company_id,
            "total": "44.00",
            "payout_type": "cash",
            "expected_payout_date": "2026-02-01",
            "expected_transfer_date": null,
            "transfer_initiated": true,
            "payment_received": true,
            "transfer_received": false,
            "tax_withholdings_covered": false
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
            .any(|error| error.get("field") == Some(&json!("transfer_initiated_forbidden")))
    );
    assert!(!errors.is_empty());
}
