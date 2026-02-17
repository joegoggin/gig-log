//! Integration tests for authentication routes.
//!
//! These tests cover core auth success and failure paths, including signup,
//! email confirmation, login, password reset verification, and password update
//! behavior with real database persistence and auth-guard enforcement.

mod support;

use std::sync::{Mutex, MutexGuard, OnceLock};

use actix_web::{App, http::StatusCode, test, web};
use serde_json::json;
use sqlx::{Pool, Postgres};
use support::{MockEmailKind, app_state_with_mock_email, test_pool, unique_email};
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

async fn user_id_for_email(pool: &Pool<Postgres>, email: &str) -> Uuid {
    sqlx::query_scalar("SELECT id FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("user should exist")
}

async fn email_confirmed_for_user(pool: &Pool<Postgres>, email: &str) -> bool {
    sqlx::query_scalar("SELECT email_confirmed FROM users WHERE email = $1")
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("user should exist")
}

async fn active_refresh_token_count(pool: &Pool<Postgres>, user_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM refresh_tokens WHERE user_id = $1 AND revoked = false",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .expect("count query should succeed")
}

async fn revoked_refresh_token_count(pool: &Pool<Postgres>, user_id: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM refresh_tokens WHERE user_id = $1 AND revoked = true",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .expect("count query should succeed")
}

#[actix_web::test]
// Verifies signup persists the user/auth code and triggers confirmation email delivery.
async fn sign_up_creates_user_auth_code_and_sends_confirmation_email() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("signup-success");
    let request = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(response).await;
    let user_id = Uuid::parse_str(
        body.get("user_id")
            .and_then(|value| value.as_str())
            .expect("response should include user_id"),
    )
    .expect("user_id should be a valid UUID");

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM auth_codes WHERE user_id = $1 AND code_type = $2::auth_code_type AND used = false",
    )
    .bind(user_id)
    .bind("email_confirmation")
    .fetch_one(&pool)
    .await
    .expect("auth code count query should succeed");

    assert_eq!(count, 1);

    let calls = mock_email.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].kind, MockEmailKind::Confirmation);
    assert_eq!(calls[0].to_email, email);
    assert_eq!(calls[0].code.len(), 6);
}

#[actix_web::test]
// Verifies duplicate signup attempts return a conflict error code.
async fn sign_up_duplicate_email_returns_conflict() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("signup-duplicate");
    let payload = json!({
        "first_name": "Taylor",
        "last_name": "User",
        "email": email,
        "password": "password123",
        "confirm": "password123"
    });

    let first = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(payload.clone())
        .to_request();
    let first_response = test::call_service(&app, first).await;
    assert_eq!(first_response.status(), StatusCode::CREATED);

    let second = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(payload)
        .to_request();
    let second_response = test::call_service(&app, second).await;
    assert_eq!(second_response.status(), StatusCode::CONFLICT);

    let body: serde_json::Value = test::read_body_json(second_response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("EMAIL_ALREADY_EXISTS")
    );
}

#[actix_web::test]
// Verifies signup payload validation failures return the standardized error array.
async fn sign_up_validation_errors_return_bad_request_shape() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let request = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "",
            "last_name": "",
            "email": "invalid-email",
            "password": "short",
            "confirm": ""
        }))
        .to_request();

    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(response).await;
    let errors = body
        .get("errors")
        .and_then(|value| value.as_array())
        .expect("validation errors should be returned as an array");

    assert!(
        errors
            .iter()
            .any(|error| error.get("field") == Some(&json!("first_name")))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.get("field") == Some(&json!("email")))
    );
}

#[actix_web::test]
// Verifies the authenticated profile endpoint rejects requests without access cookies.
async fn current_user_without_access_cookie_returns_unauthorized() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let request = test::TestRequest::get().uri("/auth/me").to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("UNAUTHORIZED")
    );
}

#[actix_web::test]
// Verifies set-password rejects requests that do not include an access token cookie.
async fn set_password_without_access_cookie_returns_unauthorized() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let request = test::TestRequest::post()
        .uri("/auth/set-password")
        .set_json(json!({
            "password": "new-password-123",
            "confirm": "new-password-123"
        }))
        .to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("UNAUTHORIZED")
    );
}

#[actix_web::test]
// Verifies set-password rejects malformed access tokens with a token-invalid error.
async fn set_password_with_invalid_access_token_returns_unauthorized() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let request = test::TestRequest::post()
        .uri("/auth/set-password")
        .insert_header(("Cookie", "access_token=not-a-jwt"))
        .set_json(json!({
            "password": "new-password-123",
            "confirm": "new-password-123"
        }))
        .to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("TOKEN_INVALID")
    );
}

#[actix_web::test]
// Verifies set-password rejects signed access tokens when no refresh-session cookie is present.
async fn set_password_with_only_access_token_returns_unauthorized() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let jwt_secret = state.env.jwt_secret.clone();
    let access_token_expiry = state.env.jwt_access_token_expiry_seconds;
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let access_token = create_access_token(
        Uuid::new_v4(),
        "ghost-user@gig-log.dev",
        &jwt_secret,
        access_token_expiry,
    )
    .expect("access token should be created for test");

    let request = test::TestRequest::post()
        .uri("/auth/set-password")
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .set_json(json!({
            "password": "new-password-123",
            "confirm": "new-password-123"
        }))
        .to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("UNAUTHORIZED")
    );
}

#[actix_web::test]
// Verifies email confirmation marks both user state and auth code usage in the database.
async fn confirm_email_marks_email_as_confirmed_and_auth_code_as_used() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("confirm-email");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == email)
        .map(|call| call.code)
        .expect("confirmation email should be captured");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": email,
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let confirmed = email_confirmed_for_user(&pool, &email).await;
    assert!(confirmed);

    let user_id = user_id_for_email(&pool, &email).await;
    let used_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM auth_codes WHERE user_id = $1 AND code_type = $2::auth_code_type AND used = true",
    )
    .bind(user_id)
    .bind("email_confirmation")
    .fetch_one(&pool)
    .await
    .expect("auth code usage query should succeed");

    assert_eq!(used_count, 1);
}

#[actix_web::test]
// Verifies login is blocked for accounts that have not confirmed email yet.
async fn log_in_requires_confirmed_email() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("login-unconfirmed");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": email,
            "password": "password123"
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::FORBIDDEN);

    let body: serde_json::Value = test::read_body_json(login_response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("EMAIL_NOT_CONFIRMED")
    );
}

#[actix_web::test]
// Verifies successful login issues both auth cookies and stores an active refresh token.
async fn log_in_sets_auth_cookies_and_persists_refresh_token() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("login-success");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == email)
        .map(|call| call.code)
        .expect("confirmation email should be captured");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": email,
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": email,
            "password": "password123"
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let cookie_names: Vec<String> = login_response
        .response()
        .cookies()
        .map(|cookie| cookie.name().to_string())
        .collect();
    assert!(cookie_names.iter().any(|name| name == "access_token"));
    assert!(cookie_names.iter().any(|name| name == "refresh_token"));

    let user_id = user_id_for_email(&pool, &email).await;
    let refresh_count = active_refresh_token_count(&pool, user_id).await;
    assert_eq!(refresh_count, 1);

    let refresh_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "refresh_token")
        .expect("refresh cookie should be present");
    assert_eq!(refresh_cookie.max_age(), None);
}

#[actix_web::test]
// Verifies remember-me login sets a persistent refresh cookie.
async fn log_in_with_remember_me_sets_persistent_refresh_cookie() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("login-remember-me");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == email)
        .map(|call| call.code)
        .expect("confirmation email should be captured");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": email,
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": email,
            "password": "password123",
            "remember_me": true
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let refresh_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "refresh_token")
        .expect("refresh cookie should be present");

    assert!(refresh_cookie.max_age().is_some());
}

#[actix_web::test]
// Verifies refresh endpoint rotates tokens and keeps one active refresh session.
async fn refresh_session_rotates_refresh_token_state() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("refresh-session");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == email)
        .map(|call| call.code)
        .expect("confirmation email should be captured");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": email,
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": email,
            "password": "password123",
            "remember_me": true
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let refresh_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "refresh_token")
        .map(|cookie| cookie.to_owned())
        .expect("refresh cookie should be set on login");

    let refresh = test::TestRequest::post()
        .uri("/auth/refresh")
        .cookie(refresh_cookie)
        .to_request();
    let refresh_response = test::call_service(&app, refresh).await;
    assert_eq!(refresh_response.status(), StatusCode::OK);

    let user_id = user_id_for_email(&pool, &email).await;
    let active_count = active_refresh_token_count(&pool, user_id).await;
    let revoked_count = revoked_refresh_token_count(&pool, user_id).await;

    assert_eq!(active_count, 1);
    assert!(revoked_count >= 1);
}

#[actix_web::test]
// Verifies refresh endpoint rejects requests without refresh cookie.
async fn refresh_session_requires_refresh_cookie() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, _mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let refresh = test::TestRequest::post().uri("/auth/refresh").to_request();
    let refresh_response = test::call_service(&app, refresh).await;

    assert_eq!(refresh_response.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
// Verifies auth endpoints normalize email case across sign-up, confirm, and login.
async fn auth_flow_normalizes_email_case_across_auth_steps() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let normalized_email = unique_email("email-normalization");
    let mixed_case_email = normalized_email.to_uppercase();

    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": mixed_case_email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == normalized_email)
        .map(|call| call.code)
        .expect("confirmation email should be captured for normalized email");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": normalized_email.to_uppercase(),
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": normalized_email.to_uppercase(),
            "password": "password123"
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let user_id = user_id_for_email(&pool, &normalized_email).await;
    let refresh_count = active_refresh_token_count(&pool, user_id).await;
    assert_eq!(refresh_count, 1);
}

#[actix_web::test]
// Verifies forgot-password responses remain generic for unknown emails (anti-enumeration).
async fn forgot_password_unknown_email_returns_generic_success_message() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("forgot-unknown");
    let request = test::TestRequest::post()
        .uri("/auth/forgot-password")
        .set_json(json!({ "email": email }))
        .to_request();
    let response = test::call_service(&app, request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        body.get("message").and_then(|message| message.as_str()),
        Some("If an account with this email exists, a password reset code has been sent.")
    );
    assert!(mock_email.calls().is_empty());
}

#[actix_web::test]
// Verifies reset-code verification marks code as used and issues auth cookies.
async fn verify_forgot_password_issues_cookies_and_marks_reset_code_used() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("verify-reset");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let forgot = test::TestRequest::post()
        .uri("/auth/forgot-password")
        .set_json(json!({ "email": email }))
        .to_request();
    let forgot_response = test::call_service(&app, forgot).await;
    assert_eq!(forgot_response.status(), StatusCode::OK);

    let reset_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::PasswordReset && call.to_email == email)
        .map(|call| call.code)
        .expect("password reset email should be captured");

    let verify = test::TestRequest::post()
        .uri("/auth/verify-forgot-password")
        .set_json(json!({
            "email": email,
            "auth_code": reset_code
        }))
        .to_request();
    let verify_response = test::call_service(&app, verify).await;
    assert_eq!(verify_response.status(), StatusCode::OK);

    let cookie_names: Vec<String> = verify_response
        .response()
        .cookies()
        .map(|cookie| cookie.name().to_string())
        .collect();
    assert!(cookie_names.iter().any(|name| name == "access_token"));
    assert!(cookie_names.iter().any(|name| name == "refresh_token"));

    let user_id = user_id_for_email(&pool, &email).await;
    let used_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::bigint FROM auth_codes WHERE user_id = $1 AND code_type = $2::auth_code_type AND used = true",
    )
    .bind(user_id)
    .bind("password_reset")
    .fetch_one(&pool)
    .await
    .expect("auth code usage query should succeed");
    assert_eq!(used_count, 1);
}

#[actix_web::test]
// Verifies password reset replaces password hash and rotates refresh token state.
async fn set_password_revokes_old_refresh_tokens_and_stores_new_token() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("set-password");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == email)
        .map(|call| call.code)
        .expect("confirmation email should be captured");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": email,
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": email,
            "password": "password123"
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let access_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "access_token")
        .map(|cookie| cookie.to_owned())
        .expect("access cookie should be set on login");
    let refresh_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "refresh_token")
        .map(|cookie| cookie.to_owned())
        .expect("refresh cookie should be set on login");

    let user_id = user_id_for_email(&pool, &email).await;
    let old_hash: String = sqlx::query_scalar("SELECT hashed_password FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("user password hash should be queryable");

    let set_password = test::TestRequest::post()
        .uri("/auth/set-password")
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .set_json(json!({
            "password": "new-password-123",
            "confirm": "new-password-123"
        }))
        .to_request();
    let set_password_response = test::call_service(&app, set_password).await;
    assert_eq!(set_password_response.status(), StatusCode::OK);

    let new_hash: String = sqlx::query_scalar("SELECT hashed_password FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("user password hash should be queryable");
    assert_ne!(old_hash, new_hash);

    let active_count = active_refresh_token_count(&pool, user_id).await;
    let revoked_count = revoked_refresh_token_count(&pool, user_id).await;

    assert_eq!(active_count, 1);
    assert!(revoked_count >= 1);
}

#[actix_web::test]
// Verifies logout invalidates the session so stale auth cookies cannot set a new password.
async fn log_out_then_set_password_with_old_cookies_returns_unauthorized() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("logout-set-password");
    let sign_up = test::TestRequest::post()
        .uri("/auth/sign-up")
        .set_json(json!({
            "first_name": "Taylor",
            "last_name": "User",
            "email": email,
            "password": "password123",
            "confirm": "password123"
        }))
        .to_request();
    let sign_up_response = test::call_service(&app, sign_up).await;
    assert_eq!(sign_up_response.status(), StatusCode::CREATED);

    let confirmation_code = mock_email
        .calls()
        .into_iter()
        .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == email)
        .map(|call| call.code)
        .expect("confirmation email should be captured");

    let confirm = test::TestRequest::post()
        .uri("/auth/confirm-email")
        .set_json(json!({
            "email": email,
            "auth_code": confirmation_code
        }))
        .to_request();
    let confirm_response = test::call_service(&app, confirm).await;
    assert_eq!(confirm_response.status(), StatusCode::OK);

    let login = test::TestRequest::post()
        .uri("/auth/log-in")
        .set_json(json!({
            "email": email,
            "password": "password123"
        }))
        .to_request();
    let login_response = test::call_service(&app, login).await;
    assert_eq!(login_response.status(), StatusCode::OK);

    let access_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "access_token")
        .map(|cookie| cookie.to_owned())
        .expect("access cookie should be set on login");
    let refresh_cookie = login_response
        .response()
        .cookies()
        .find(|cookie| cookie.name() == "refresh_token")
        .map(|cookie| cookie.to_owned())
        .expect("refresh cookie should be set on login");

    let log_out = test::TestRequest::post()
        .uri("/auth/log-out")
        .cookie(refresh_cookie.clone())
        .to_request();
    let log_out_response = test::call_service(&app, log_out).await;
    assert_eq!(log_out_response.status(), StatusCode::OK);

    let set_password = test::TestRequest::post()
        .uri("/auth/set-password")
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .set_json(json!({
            "password": "new-password-123",
            "confirm": "new-password-123"
        }))
        .to_request();
    let set_password_response = test::call_service(&app, set_password).await;
    assert_eq!(set_password_response.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(set_password_response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|code| code.as_str()),
        Some("UNAUTHORIZED")
    );
}
