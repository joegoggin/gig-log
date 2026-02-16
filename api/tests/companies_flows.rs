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
    assert!(!errors.is_empty());
}

#[actix_web::test]
// Verifies company detail responses include Phoenix-style aggregates and paginated lists.
async fn get_company_returns_paginated_jobs_payments_and_aggregates() {
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

    let email = unique_email("companies-detail");
    let user_id = insert_user(&pool, &email).await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let create_request = test::TestRequest::post()
        .uri("/companies")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({
            "name": "Detail Company",
            "requires_tax_withholdings": true,
            "tax_withholding_rate": "30.00"
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

    let mut inserted_job_ids = Vec::new();
    for index in 0..6 {
        let title = format!("Job {}", index + 1);
        let job_id: Uuid = sqlx::query_scalar(
            r#"
            INSERT INTO jobs (
                company_id,
                user_id,
                title,
                payment_type,
                hourly_rate,
                updated_at
            )
            VALUES ($1, $2, $3, 'hourly', 25.00, NOW() + (($4::TEXT || ' seconds')::INTERVAL))
            RETURNING id
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .bind(title)
        .bind(index as i64)
        .fetch_one(&pool)
        .await
        .expect("job should be inserted");

        inserted_job_ids.push(job_id);
    }

    for index in 0..6 {
        let payout_type = if index % 2 == 0 { "paypal" } else { "cash" };
        let payment_received = index % 2 == 0;
        let transfer_received = index % 3 == 0;

        sqlx::query(
            r#"
            INSERT INTO payments (
                user_id,
                company_id,
                total,
                payout_type,
                payment_received,
                transfer_received,
                updated_at
            )
            VALUES ($1, $2, $3, $4::payout_type_enum, $5, $6, NOW() + (($7::TEXT || ' seconds')::INTERVAL))
            "#,
        )
        .bind(user_id)
        .bind(company_id)
        .bind((index + 1) as i64 * 10)
        .bind(payout_type)
        .bind(payment_received)
        .bind(transfer_received)
        .bind(index as i64)
        .execute(&pool)
        .await
        .expect("payment should be inserted");
    }

    sqlx::query(
        r#"
        INSERT INTO work_sessions (
            user_id,
            job_id,
            start_time,
            end_time,
            accumulated_paused_duration,
            time_reported
        )
        VALUES
            ($1, $2, NOW() - INTERVAL '3 hours', NOW() - INTERVAL '1 hour', 600, true),
            ($1, $3, NOW() - INTERVAL '90 minutes', NOW(), 0, true)
        "#,
    )
    .bind(user_id)
    .bind(inserted_job_ids[0])
    .bind(inserted_job_ids[1])
    .execute(&pool)
    .await
    .expect("work sessions should be inserted");

    let detail_request = test::TestRequest::get()
        .uri(&format!("/companies/{company_id}"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let detail_response = test::call_service(&app, detail_request).await;
    assert_eq!(detail_response.status(), StatusCode::OK);

    let detail_body: serde_json::Value = test::read_body_json(detail_response).await;
    assert_eq!(
        detail_body
            .get("jobs_has_more")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        detail_body
            .get("payments_has_more")
            .and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        detail_body
            .get("paginated_jobs")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(5)
    );
    assert_eq!(
        detail_body
            .get("paginated_payments")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(5)
    );
    assert_eq!(
        detail_body
            .get("company")
            .and_then(|company| company.get("hours"))
            .and_then(|value| value.as_str()),
        Some("3h 20m")
    );

    let payment_total = detail_body
        .get("company")
        .and_then(|company| company.get("payment_total"))
        .expect("company payment_total should exist");
    if let Some(total_str) = payment_total.as_str() {
        assert!(total_str.starts_with("210"));
    } else {
        assert_eq!(payment_total.as_i64(), Some(210));
    }

    let jobs_page_2_request = test::TestRequest::get()
        .uri(&format!("/companies/{company_id}?jobs_page=2"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let jobs_page_2_response = test::call_service(&app, jobs_page_2_request).await;
    assert_eq!(jobs_page_2_response.status(), StatusCode::OK);
    let jobs_page_2_body: serde_json::Value = test::read_body_json(jobs_page_2_response).await;
    assert_eq!(
        jobs_page_2_body
            .get("jobs_has_more")
            .and_then(|value| value.as_bool()),
        Some(false)
    );
    assert_eq!(
        jobs_page_2_body
            .get("paginated_jobs")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(1)
    );
    assert_eq!(
        jobs_page_2_body
            .get("paginated_payments")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(5)
    );

    let payments_page_2_request = test::TestRequest::get()
        .uri(&format!("/companies/{company_id}?payments_page=2"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let payments_page_2_response = test::call_service(&app, payments_page_2_request).await;
    assert_eq!(payments_page_2_response.status(), StatusCode::OK);
    let payments_page_2_body: serde_json::Value =
        test::read_body_json(payments_page_2_response).await;
    assert_eq!(
        payments_page_2_body
            .get("payments_has_more")
            .and_then(|value| value.as_bool()),
        Some(false)
    );
    assert_eq!(
        payments_page_2_body
            .get("paginated_payments")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(1)
    );
    assert_eq!(
        payments_page_2_body
            .get("paginated_jobs")
            .and_then(|value| value.as_array())
            .map(std::vec::Vec::len),
        Some(5)
    );
}
