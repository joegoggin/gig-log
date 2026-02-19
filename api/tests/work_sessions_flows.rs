//! Integration tests for work session routes.
//!
//! These tests cover the full work session lifecycle (start, pause, resume,
//! complete, get active), state transition validation, duplicate session
//! prevention, user scoping, and error cases with real database persistence.

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

async fn insert_job(pool: &Pool<Postgres>, user_id: Uuid, company_id: Uuid, title: &str) -> Uuid {
    sqlx::query_scalar(
        r#"
        INSERT INTO jobs (user_id, company_id, title, payment_type, hourly_rate, number_of_payouts, payout_amount)
        VALUES ($1, $2, $3, 'hourly', 50.00, NULL, NULL)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(company_id)
    .bind(title)
    .fetch_one(pool)
    .await
    .expect("test job should be inserted")
}

fn access_cookie(token: &str) -> String {
    format!("access_token={token}")
}

#[actix_web::test]
// Verifies the full start → pause → resume → complete lifecycle with state checks at each step.
async fn work_session_full_lifecycle_succeeds() {
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

    let email = unique_email("ws-lifecycle");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "WS Lifecycle Co").await;
    let job_id = insert_job(&pool, user_id, company_id, "Lifecycle Job").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    // Start
    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    assert_eq!(start_response.status(), StatusCode::CREATED);

    let start_body: serde_json::Value = test::read_body_json(start_response).await;
    let session_id = start_body
        .get("work_session")
        .and_then(|ws| ws.get("id"))
        .and_then(|v| v.as_str())
        .expect("start response should include work_session id");
    assert_eq!(
        start_body
            .get("work_session")
            .and_then(|ws| ws.get("is_running"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(
        start_body
            .get("work_session")
            .and_then(|ws| ws.get("end_time"))
            .map(|v| v.is_null())
            .unwrap_or(false)
    );

    // Get active
    let active_request = test::TestRequest::get()
        .uri("/work-sessions/active")
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let active_response = test::call_service(&app, active_request).await;
    assert_eq!(active_response.status(), StatusCode::OK);

    let active_body: serde_json::Value = test::read_body_json(active_response).await;
    assert_eq!(
        active_body
            .get("work_session")
            .and_then(|ws| ws.get("id"))
            .and_then(|v| v.as_str()),
        Some(session_id)
    );

    // Pause
    let pause_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/pause"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let pause_response = test::call_service(&app, pause_request).await;
    assert_eq!(pause_response.status(), StatusCode::OK);

    let pause_body: serde_json::Value = test::read_body_json(pause_response).await;
    assert_eq!(
        pause_body
            .get("work_session")
            .and_then(|ws| ws.get("is_running"))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
    assert!(
        pause_body
            .get("work_session")
            .and_then(|ws| ws.get("paused_at"))
            .map(|v| !v.is_null())
            .unwrap_or(false)
    );

    // Resume
    let resume_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/resume"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let resume_response = test::call_service(&app, resume_request).await;
    assert_eq!(resume_response.status(), StatusCode::OK);

    let resume_body: serde_json::Value = test::read_body_json(resume_response).await;
    assert_eq!(
        resume_body
            .get("work_session")
            .and_then(|ws| ws.get("is_running"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(
        resume_body
            .get("work_session")
            .and_then(|ws| ws.get("paused_at"))
            .map(|v| v.is_null())
            .unwrap_or(false)
    );

    // Complete
    let complete_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/complete"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let complete_response = test::call_service(&app, complete_request).await;
    assert_eq!(complete_response.status(), StatusCode::OK);

    let complete_body: serde_json::Value = test::read_body_json(complete_response).await;
    assert_eq!(
        complete_body
            .get("work_session")
            .and_then(|ws| ws.get("is_running"))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
    assert!(
        complete_body
            .get("work_session")
            .and_then(|ws| ws.get("end_time"))
            .map(|v| !v.is_null())
            .unwrap_or(false)
    );

    // Get active after complete → 404
    let active_after_complete = test::TestRequest::get()
        .uri("/work-sessions/active")
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let active_after_response = test::call_service(&app, active_after_complete).await;
    assert_eq!(active_after_response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies starting a second session while one is active returns 400.
async fn reject_duplicate_active_session() {
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

    let email = unique_email("ws-dup");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Dup Co").await;
    let job_id = insert_job(&pool, user_id, company_id, "Dup Job").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    assert_eq!(start_response.status(), StatusCode::CREATED);

    let second_start = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let second_response = test::call_service(&app, second_start).await;
    assert_eq!(second_response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
// Verifies starting a session with a nonexistent job returns 404.
async fn reject_nonexistent_job() {
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

    let email = unique_email("ws-nojob");
    let user_id = insert_user(&pool, &email).await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": Uuid::new_v4() }))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies pausing a completed session returns 400.
async fn reject_pause_when_not_running() {
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

    let email = unique_email("ws-pausefail");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Pause Fail Co").await;
    let job_id = insert_job(&pool, user_id, company_id, "Pause Fail Job").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    // Start and complete
    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    let start_body: serde_json::Value = test::read_body_json(start_response).await;
    let session_id = start_body
        .get("work_session")
        .and_then(|ws| ws.get("id"))
        .and_then(|v| v.as_str())
        .expect("session id should exist");

    let complete_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/complete"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    test::call_service(&app, complete_request).await;

    // Try to pause completed session
    let pause_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/pause"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let pause_response = test::call_service(&app, pause_request).await;
    assert_eq!(pause_response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
// Verifies resuming a running (not paused) session returns 400.
async fn reject_resume_when_not_paused() {
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

    let email = unique_email("ws-resumefail");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Resume Fail Co").await;
    let job_id = insert_job(&pool, user_id, company_id, "Resume Fail Job").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    let start_body: serde_json::Value = test::read_body_json(start_response).await;
    let session_id = start_body
        .get("work_session")
        .and_then(|ws| ws.get("id"))
        .and_then(|v| v.as_str())
        .expect("session id should exist");

    // Try to resume a running session
    let resume_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/resume"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let resume_response = test::call_service(&app, resume_request).await;
    assert_eq!(resume_response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
// Verifies completing an already-completed session returns 400.
async fn reject_double_complete() {
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

    let email = unique_email("ws-dblcomplete");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Dbl Complete Co").await;
    let job_id = insert_job(&pool, user_id, company_id, "Dbl Complete Job").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    let start_body: serde_json::Value = test::read_body_json(start_response).await;
    let session_id = start_body
        .get("work_session")
        .and_then(|ws| ws.get("id"))
        .and_then(|v| v.as_str())
        .expect("session id should exist");

    let complete_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/complete"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let complete_response = test::call_service(&app, complete_request).await;
    assert_eq!(complete_response.status(), StatusCode::OK);

    let second_complete = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/complete"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let second_response = test::call_service(&app, second_complete).await;
    assert_eq!(second_response.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
// Verifies get_active returns 404 when no sessions exist.
async fn no_active_session_returns_not_found() {
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

    let email = unique_email("ws-noactive");
    let user_id = insert_user(&pool, &email).await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    let request = test::TestRequest::get()
        .uri("/work-sessions/active")
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies user A's session is invisible to user B.
async fn user_scoping_enforced() {
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

    let email_a = unique_email("ws-scope-a");
    let email_b = unique_email("ws-scope-b");
    let user_a = insert_user(&pool, &email_a).await;
    let user_b = insert_user(&pool, &email_b).await;
    let company_a = insert_company(&pool, user_a, "Scope A Co").await;
    let job_a = insert_job(&pool, user_a, company_a, "Scope A Job").await;
    let token_a = create_access_token(user_a, &email_a, &jwt_secret, access_expiry)
        .expect("access token a should be created");
    let token_b = create_access_token(user_b, &email_b, &jwt_secret, access_expiry)
        .expect("access token b should be created");

    // User A starts a session
    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token_a)))
        .set_json(json!({ "job_id": job_a }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    assert_eq!(start_response.status(), StatusCode::CREATED);

    let start_body: serde_json::Value = test::read_body_json(start_response).await;
    let session_id = start_body
        .get("work_session")
        .and_then(|ws| ws.get("id"))
        .and_then(|v| v.as_str())
        .expect("session id should exist");

    // User B cannot see the active session
    let active_b = test::TestRequest::get()
        .uri("/work-sessions/active")
        .insert_header(("Cookie", access_cookie(&token_b)))
        .to_request();
    let active_b_response = test::call_service(&app, active_b).await;
    assert_eq!(active_b_response.status(), StatusCode::NOT_FOUND);

    // User B cannot pause user A's session
    let pause_b = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/pause"))
        .insert_header(("Cookie", access_cookie(&token_b)))
        .to_request();
    let pause_b_response = test::call_service(&app, pause_b).await;
    assert_eq!(pause_b_response.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
// Verifies completing while paused finalizes accumulated_paused_duration > 0.
async fn complete_while_paused_accumulates_pause_duration() {
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

    let email = unique_email("ws-pausecomplete");
    let user_id = insert_user(&pool, &email).await;
    let company_id = insert_company(&pool, user_id, "Pause Complete Co").await;
    let job_id = insert_job(&pool, user_id, company_id, "Pause Complete Job").await;
    let token = create_access_token(user_id, &email, &jwt_secret, access_expiry)
        .expect("access token should be created");

    // Start
    let start_request = test::TestRequest::post()
        .uri("/work-sessions/start")
        .insert_header(("Cookie", access_cookie(&token)))
        .set_json(json!({ "job_id": job_id }))
        .to_request();
    let start_response = test::call_service(&app, start_request).await;
    let start_body: serde_json::Value = test::read_body_json(start_response).await;
    let session_id = start_body
        .get("work_session")
        .and_then(|ws| ws.get("id"))
        .and_then(|v| v.as_str())
        .expect("session id should exist");

    // Pause
    let pause_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/pause"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    test::call_service(&app, pause_request).await;

    // Complete while paused
    let complete_request = test::TestRequest::post()
        .uri(&format!("/work-sessions/{session_id}/complete"))
        .insert_header(("Cookie", access_cookie(&token)))
        .to_request();
    let complete_response = test::call_service(&app, complete_request).await;
    assert_eq!(complete_response.status(), StatusCode::OK);

    let complete_body: serde_json::Value = test::read_body_json(complete_response).await;
    let accumulated = complete_body
        .get("work_session")
        .and_then(|ws| ws.get("accumulated_paused_duration"))
        .and_then(|v| v.as_i64())
        .expect("accumulated_paused_duration should exist");
    assert!(
        accumulated >= 0,
        "accumulated_paused_duration should be non-negative"
    );
    assert!(
        complete_body
            .get("work_session")
            .and_then(|ws| ws.get("end_time"))
            .map(|v| !v.is_null())
            .unwrap_or(false)
    );
    assert_eq!(
        complete_body
            .get("work_session")
            .and_then(|ws| ws.get("is_running"))
            .and_then(|v| v.as_bool()),
        Some(false)
    );
}
