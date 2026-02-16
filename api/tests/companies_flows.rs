//! Integration tests for company routes.
//!
//! These tests cover authenticated company CRUD flows, user-scoped access
//! controls, and request validation behavior with real database persistence.

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

fn access_cookie(token: &str) -> String {
    format!("access_token={token}")
}

#[actix_web::test]
// Verifies the full authenticated CRUD lifecycle for a company.
async fn companies_crud_flow_succeeds_for_owner() {
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

    let email = unique_email("companies-owner");
    let user_id = insert_user(&pool, &email).await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/companies")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "name": "Acme Studio",
            "requires_tax_withholdings": false,
            "tax_withholding_rate": null
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let company_id = Uuid::parse_str(
        create_body
            .get("company")
            .and_then(|company| company.get("id"))
            .and_then(|value| value.as_str())
            .expect("create response should include company id"),
    )
    .expect("company id should be a UUID");

    let list_request = test::TestRequest::get()
        .uri("/companies")
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let list_response = test::call_service(&app, list_request).await;
    assert_eq!(list_response.status(), StatusCode::OK);

    let list_body: serde_json::Value = test::read_body_json(list_response).await;
    let companies = list_body
        .get("companies")
        .and_then(|value| value.as_array())
        .expect("companies list should be an array");
    assert_eq!(companies.len(), 1);

    let get_request = test::TestRequest::get()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let get_response = test::call_service(&app, get_request).await;
    assert_eq!(get_response.status(), StatusCode::OK);

    let update_request = test::TestRequest::put()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "name": "Acme Studio Updated",
            "requires_tax_withholdings": true,
            "tax_withholding_rate": "30.00"
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_response).await;
    assert_eq!(
        update_body
            .get("company")
            .and_then(|company| company.get("name"))
            .and_then(|value| value.as_str()),
        Some("Acme Studio Updated")
    );

    let delete_request = test::TestRequest::delete()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let delete_response = test::call_service(&app, delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::OK);

    let get_deleted_request = test::TestRequest::get()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let get_deleted_response = test::call_service(&app, get_deleted_request).await;
    assert_eq!(get_deleted_response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies users cannot access or mutate companies they do not own.
async fn company_routes_enforce_user_scoping() {
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

    let owner_email = unique_email("companies-owner-scope");
    let other_email = unique_email("companies-other-scope");
    let owner_id = insert_user(&pool, &owner_email).await;
    let other_id = insert_user(&pool, &other_email).await;
    let owner_token = create_access_token(owner_id, &owner_email, &jwt_secret, access_expiry)
        .expect("owner access token should be created");
    let other_token = create_access_token(other_id, &other_email, &jwt_secret, access_expiry)
        .expect("other access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/companies")
        .insert_header(("Cookie", access_cookie(&owner_token)))
        .set_json(json!({
            "name": "Scoped Company",
            "requires_tax_withholdings": false,
            "tax_withholding_rate": null
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let company_id = create_body
        .get("company")
        .and_then(|company| company.get("id"))
        .and_then(|value| value.as_str())
        .expect("created company id should exist");

    let get_request = test::TestRequest::get()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .to_request();
    let get_response = test::call_service(&app, get_request).await;
    assert_eq!(get_response.status(), StatusCode::NOT_FOUND);

    let update_request = test::TestRequest::put()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .set_json(json!({
            "name": "Intrusion Update",
            "requires_tax_withholdings": false,
            "tax_withholding_rate": null
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);

    let delete_request = test::TestRequest::delete()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&other_token)))
        .to_request();
    let delete_response = test::call_service(&app, delete_request).await;
    assert_eq!(delete_response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies invalid company payloads return the standardized validation-error shape.
async fn create_company_with_invalid_payload_returns_validation_errors() {
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

    let email = unique_email("companies-validation");
    let user_id = insert_user(&pool, &email).await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let request = test::TestRequest::post()
        .uri("/companies")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "name": "",
            "requires_tax_withholdings": true,
            "tax_withholding_rate": null
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
            .any(|error| error.get("field") == Some(&json!("name")))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.get("field") == Some(&json!("tax_withholding_rate_required")))
    );
}
