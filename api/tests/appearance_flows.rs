//! Integration tests for appearance customization routes.
//!
//! These tests cover custom palette creation, editing, and active palette
//! persistence, including default behavior, success flows, and ownership
//! enforcement.

mod support;

use std::sync::{Mutex, MutexGuard, OnceLock};

use actix_web::{App, http::StatusCode, test, web};
use serde_json::json;
use sqlx::{Pool, Postgres};
use support::{MockEmailKind, app_state_with_mock_email, test_pool, unique_email};
use uuid::Uuid;

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

macro_rules! sign_up_confirm_and_log_in {
    ($app:expr, $mock_email:expr, $email:expr) => {{
        let sign_up = test::TestRequest::post()
            .uri("/auth/sign-up")
            .set_json(json!({
                "first_name": "Taylor",
                "last_name": "User",
                "email": $email,
                "password": "password123",
                "confirm": "password123"
            }))
            .to_request();
        let sign_up_response = test::call_service($app, sign_up).await;
        assert_eq!(sign_up_response.status(), StatusCode::CREATED);

        let confirmation_code = $mock_email
            .calls()
            .into_iter()
            .find(|call| call.kind == MockEmailKind::Confirmation && call.to_email == $email)
            .map(|call| call.code)
            .expect("confirmation email should be captured");

        let confirm = test::TestRequest::post()
            .uri("/auth/confirm-email")
            .set_json(json!({
                "email": $email,
                "auth_code": confirmation_code
            }))
            .to_request();
        let confirm_response = test::call_service($app, confirm).await;
        assert_eq!(confirm_response.status(), StatusCode::OK);

        let log_in = test::TestRequest::post()
            .uri("/auth/log-in")
            .set_json(json!({
                "email": $email,
                "password": "password123"
            }))
            .to_request();
        let log_in_response = test::call_service($app, log_in).await;
        assert_eq!(log_in_response.status(), StatusCode::OK);

        let access_token = log_in_response
            .response()
            .cookies()
            .find(|cookie| cookie.name() == "access_token")
            .map(|cookie| cookie.value().to_string())
            .expect("access cookie should be present");
        let refresh_token = log_in_response
            .response()
            .cookies()
            .find(|cookie| cookie.name() == "refresh_token")
            .map(|cookie| cookie.value().to_string())
            .expect("refresh cookie should be present");

        (access_token, refresh_token)
    }};
}

#[actix_web::test]
// Verifies default appearance response when user has no persisted palette preference.
async fn get_appearance_returns_default_active_preset_without_preferences() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("appearance-default");
    let (access_token, _) = sign_up_confirm_and_log_in!(&app, &mock_email, email.as_str());

    let request = test::TestRequest::get()
        .uri("/appearance")
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .to_request();
    let response = test::call_service(&app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(
        body.get("active_palette")
            .and_then(|value| value.get("palette_type"))
            .and_then(|value| value.as_str()),
        Some("preset")
    );
    assert_eq!(
        body.get("active_palette")
            .and_then(|value| value.get("preset_palette"))
            .and_then(|value| value.as_str()),
        Some("default")
    );
    assert_eq!(
        body.get("custom_palettes")
            .and_then(|value| value.as_array())
            .map(|value| value.len()),
        Some(0)
    );
}

#[actix_web::test]
// Verifies custom palette creation persists data and sets custom selection active.
async fn create_custom_palette_persists_palette_and_sets_active_selection() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("appearance-create");
    let (access_token, _) = sign_up_confirm_and_log_in!(&app, &mock_email, email.as_str());

    let create_request = test::TestRequest::post()
        .uri("/appearance/palettes")
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .set_json(json!({
            "name": "Ocean",
            "background_seed_hex": "#a9b1d6",
            "text_seed_hex": "#1a1b26",
            "primary_seed_hex": "#9ece6a",
            "secondary_seed_hex": "#7aa2f7",
            "green_seed_hex": "#66bb6a",
            "red_seed_hex": "#e27d7c",
            "yellow_seed_hex": "#d0a761",
            "blue_seed_hex": "#5c93cd",
            "magenta_seed_hex": "#a082ce",
            "cyan_seed_hex": "#59b7aa"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let created_palette_id = Uuid::parse_str(
        create_body
            .get("palette")
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str())
            .expect("created palette should include id"),
    )
    .expect("palette id should be a valid UUID");

    assert_eq!(
        create_body
            .get("active_palette")
            .and_then(|value| value.get("palette_type"))
            .and_then(|value| value.as_str()),
        Some("custom")
    );

    let user_id = user_id_for_email(&pool, &email).await;
    let palette_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*)::bigint FROM user_color_palettes WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .expect("palette count query should succeed");
    assert_eq!(palette_count, 1);

    let preference: (String, Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT active_palette_type, active_preset_palette, active_custom_palette_id FROM user_appearance_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("preference row should exist");

    assert_eq!(preference.0, "custom");
    assert_eq!(preference.1, None);
    assert_eq!(preference.2, Some(created_palette_id));
}

#[actix_web::test]
// Verifies custom palette edits persist updated fields and regenerated tokens.
async fn update_custom_palette_persists_updated_fields() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("appearance-update");
    let (access_token, _) = sign_up_confirm_and_log_in!(&app, &mock_email, email.as_str());

    let create_request = test::TestRequest::post()
        .uri("/appearance/palettes")
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .set_json(json!({
            "name": "Ocean",
            "background_seed_hex": "#a9b1d6",
            "text_seed_hex": "#1a1b26",
            "primary_seed_hex": "#9ece6a",
            "secondary_seed_hex": "#7aa2f7",
            "green_seed_hex": "#66bb6a",
            "red_seed_hex": "#e27d7c",
            "yellow_seed_hex": "#d0a761",
            "blue_seed_hex": "#5c93cd",
            "magenta_seed_hex": "#a082ce",
            "cyan_seed_hex": "#59b7aa"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let created_palette_id = Uuid::parse_str(
        create_body
            .get("palette")
            .and_then(|value| value.get("id"))
            .and_then(|value| value.as_str())
            .expect("created palette should include id"),
    )
    .expect("palette id should be a valid UUID");

    let update_request = test::TestRequest::put()
        .uri(&format!("/appearance/palettes/{created_palette_id}"))
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .set_json(json!({
            "name": "Ocean Dusk",
            "background_seed_hex": "#f3f3f3",
            "text_seed_hex": "#101820",
            "primary_seed_hex": "#336699",
            "secondary_seed_hex": "#8e24aa",
            "green_seed_hex": "#2e7d32",
            "red_seed_hex": "#b71c1c",
            "yellow_seed_hex": "#f9a825",
            "blue_seed_hex": "#1976d2",
            "magenta_seed_hex": "#ad1457",
            "cyan_seed_hex": "#00838f"
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::OK);

    let update_body: serde_json::Value = test::read_body_json(update_response).await;
    assert_eq!(
        update_body
            .get("palette")
            .and_then(|value| value.get("name"))
            .and_then(|value| value.as_str()),
        Some("Ocean Dusk")
    );
    assert_eq!(
        update_body
            .get("palette")
            .and_then(|value| value.get("generated_tokens"))
            .and_then(|value| value.get("primary_100"))
            .and_then(|value| value.as_str()),
        Some("51, 102, 153")
    );

    let user_id = user_id_for_email(&pool, &email).await;
    let row: (String, String, serde_json::Value, i32) = sqlx::query_as(
        "SELECT name, primary_seed_hex, generated_tokens, generation_version FROM user_color_palettes WHERE id = $1 AND user_id = $2",
    )
    .bind(created_palette_id)
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("updated palette row should exist");

    assert_eq!(row.0, "Ocean Dusk");
    assert_eq!(row.1, "#336699");
    assert_eq!(
        row.2.get("primary_100").and_then(|value| value.as_str()),
        Some("51, 102, 153")
    );
    assert_eq!(row.3, 2);

    let preference: (String, Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT active_palette_type, active_preset_palette, active_custom_palette_id FROM user_appearance_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("preference row should exist");

    assert_eq!(preference.0, "custom");
    assert_eq!(preference.1, None);
    assert_eq!(preference.2, Some(created_palette_id));
}

#[actix_web::test]
// Verifies active palette can be switched back to a preset after custom creation.
async fn set_active_palette_to_preset_updates_preference_row() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool.clone());
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let email = unique_email("appearance-set-preset");
    let (access_token, _) = sign_up_confirm_and_log_in!(&app, &mock_email, email.as_str());

    let create_request = test::TestRequest::post()
        .uri("/appearance/palettes")
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .set_json(json!({
            "name": "Meadow",
            "background_seed_hex": "#a9b1d6",
            "text_seed_hex": "#1a1b26",
            "primary_seed_hex": "#9ece6a",
            "secondary_seed_hex": "#7aa2f7",
            "green_seed_hex": "#66bb6a",
            "red_seed_hex": "#e27d7c",
            "yellow_seed_hex": "#d0a761",
            "blue_seed_hex": "#5c93cd",
            "magenta_seed_hex": "#a082ce",
            "cyan_seed_hex": "#59b7aa"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let set_active_request = test::TestRequest::put()
        .uri("/appearance/active-palette")
        .insert_header(("Cookie", format!("access_token={access_token}")))
        .set_json(json!({
            "palette_type": "preset",
            "preset_palette": "forest"
        }))
        .to_request();
    let set_active_response = test::call_service(&app, set_active_request).await;
    assert_eq!(set_active_response.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(set_active_response).await;
    assert_eq!(
        body.get("active_palette")
            .and_then(|value| value.get("palette_type"))
            .and_then(|value| value.as_str()),
        Some("preset")
    );
    assert_eq!(
        body.get("active_palette")
            .and_then(|value| value.get("preset_palette"))
            .and_then(|value| value.as_str()),
        Some("forest")
    );

    let user_id = user_id_for_email(&pool, &email).await;
    let preference: (String, Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT active_palette_type, active_preset_palette, active_custom_palette_id FROM user_appearance_preferences WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .expect("preference row should exist");

    assert_eq!(preference.0, "preset");
    assert_eq!(preference.1.as_deref(), Some("forest"));
    assert_eq!(preference.2, None);
}

#[actix_web::test]
// Verifies users cannot activate custom palettes they do not own.
async fn set_active_palette_rejects_custom_palette_from_another_user() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let owner_email = unique_email("appearance-owner");
    let (owner_access_token, _) =
        sign_up_confirm_and_log_in!(&app, &mock_email, owner_email.as_str());

    let create_request = test::TestRequest::post()
        .uri("/appearance/palettes")
        .insert_header(("Cookie", format!("access_token={owner_access_token}")))
        .set_json(json!({
            "name": "Owner Palette",
            "background_seed_hex": "#a9b1d6",
            "text_seed_hex": "#1a1b26",
            "primary_seed_hex": "#9ece6a",
            "secondary_seed_hex": "#7aa2f7",
            "green_seed_hex": "#66bb6a",
            "red_seed_hex": "#e27d7c",
            "yellow_seed_hex": "#d0a761",
            "blue_seed_hex": "#5c93cd",
            "magenta_seed_hex": "#a082ce",
            "cyan_seed_hex": "#59b7aa"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let owner_palette_id = create_body
        .get("palette")
        .and_then(|value| value.get("id"))
        .and_then(|value| value.as_str())
        .expect("owner palette id should be present")
        .to_string();

    let other_email = unique_email("appearance-other");
    let (other_access_token, _) =
        sign_up_confirm_and_log_in!(&app, &mock_email, other_email.as_str());

    let set_active_request = test::TestRequest::put()
        .uri("/appearance/active-palette")
        .insert_header(("Cookie", format!("access_token={other_access_token}")))
        .set_json(json!({
            "palette_type": "custom",
            "custom_palette_id": owner_palette_id
        }))
        .to_request();
    let set_active_response = test::call_service(&app, set_active_request).await;
    assert_eq!(set_active_response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(set_active_response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|value| value.as_str()),
        Some("NOT_FOUND")
    );
}

#[actix_web::test]
// Verifies users cannot edit custom palettes they do not own.
async fn update_custom_palette_rejects_palette_owned_by_another_user() {
    let _guard = test_guard();
    let pool = test_pool().await;
    let (state, mock_email) = app_state_with_mock_email(pool);
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(state))
            .configure(configure_routes),
    )
    .await;

    let owner_email = unique_email("appearance-update-owner");
    let (owner_access_token, _) =
        sign_up_confirm_and_log_in!(&app, &mock_email, owner_email.as_str());

    let create_request = test::TestRequest::post()
        .uri("/appearance/palettes")
        .insert_header(("Cookie", format!("access_token={owner_access_token}")))
        .set_json(json!({
            "name": "Owner Palette",
            "background_seed_hex": "#a9b1d6",
            "text_seed_hex": "#1a1b26",
            "primary_seed_hex": "#9ece6a",
            "secondary_seed_hex": "#7aa2f7",
            "green_seed_hex": "#66bb6a",
            "red_seed_hex": "#e27d7c",
            "yellow_seed_hex": "#d0a761",
            "blue_seed_hex": "#5c93cd",
            "magenta_seed_hex": "#a082ce",
            "cyan_seed_hex": "#59b7aa"
        }))
        .to_request();
    let create_response = test::call_service(&app, create_request).await;
    assert_eq!(create_response.status(), StatusCode::CREATED);

    let create_body: serde_json::Value = test::read_body_json(create_response).await;
    let owner_palette_id = create_body
        .get("palette")
        .and_then(|value| value.get("id"))
        .and_then(|value| value.as_str())
        .expect("owner palette id should be present")
        .to_string();

    let other_email = unique_email("appearance-update-other");
    let (other_access_token, _) =
        sign_up_confirm_and_log_in!(&app, &mock_email, other_email.as_str());

    let update_request = test::TestRequest::put()
        .uri(&format!("/appearance/palettes/{owner_palette_id}"))
        .insert_header(("Cookie", format!("access_token={other_access_token}")))
        .set_json(json!({
            "name": "Should Fail",
            "background_seed_hex": "#f3f3f3",
            "text_seed_hex": "#101820",
            "primary_seed_hex": "#336699",
            "secondary_seed_hex": "#8e24aa",
            "green_seed_hex": "#2e7d32",
            "red_seed_hex": "#b71c1c",
            "yellow_seed_hex": "#f9a825",
            "blue_seed_hex": "#1976d2",
            "magenta_seed_hex": "#ad1457",
            "cyan_seed_hex": "#00838f"
        }))
        .to_request();
    let update_response = test::call_service(&app, update_request).await;
    assert_eq!(update_response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = test::read_body_json(update_response).await;
    assert_eq!(
        body.get("error")
            .and_then(|error| error.get("code"))
            .and_then(|value| value.as_str()),
        Some("NOT_FOUND")
    );
}
