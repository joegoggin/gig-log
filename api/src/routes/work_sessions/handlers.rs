//! HTTP handler functions for work session endpoints.
//!
//! This module contains handlers for starting, pausing, resuming, completing,
//! and retrieving active work sessions for authenticated users.

use actix_web::{HttpResponse, get, post, web};
use chrono::Utc;
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::core::app_state::AppState;
use crate::core::error::{ApiError, ApiResult};
use crate::extractors::ValidatedJson;
use crate::repository::work_sessions::{WorkSessionWriteInput, WorkSessionsRepo};

use super::payloads::{StartWorkSessionRequest, WorkSessionResponse};

/// Starts a new work session for a job.
///
/// Verifies the job exists and belongs to the authenticated user, and that
/// there is no active (uncompleted) session already running.
///
/// # Route
///
/// `POST /work-sessions/start`
///
/// # Request Body ([`StartWorkSessionRequest`])
///
/// - `job_id` - Job identifier to track time against
///
/// # Response Body ([`WorkSessionResponse`])
///
/// - `work_session` - The newly created work session
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If the job does not exist for the authenticated user
/// - `ValidationError` - If there is already an active work session
/// - `DatabaseError` - If the database operation fails
#[post("/work-sessions/start")]
pub async fn start_work_session(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    body: ValidatedJson<StartWorkSessionRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();

    let job_exists =
        WorkSessionsRepo::job_exists_for_user(&state.pool, auth_user.user_id, body.job_id).await?;

    if !job_exists {
        return Err(ApiError::NotFound("Job not found".to_string()));
    }

    let active = WorkSessionsRepo::find_active_for_user(&state.pool, auth_user.user_id).await?;

    if active.is_some() {
        return Err(ApiError::ValidationError(
            "An active work session already exists".to_string(),
        ));
    }

    let input = WorkSessionWriteInput {
        job_id: body.job_id,
        start_time: Some(Utc::now()),
        end_time: None,
        is_running: true,
        accumulated_paused_duration: 0,
        paused_at: None,
        time_reported: false,
    };

    let work_session =
        WorkSessionsRepo::create_for_user(&state.pool, auth_user.user_id, &input).await?;

    Ok(HttpResponse::Created().json(WorkSessionResponse { work_session }))
}

/// Pauses a running work session.
///
/// Sets `paused_at` to the current time and `is_running` to false.
///
/// # Route
///
/// `POST /work-sessions/{id}/pause`
///
/// # Response Body ([`WorkSessionResponse`])
///
/// - `work_session` - The updated work session
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If the work session does not exist for the authenticated user
/// - `ValidationError` - If the session is not currently running or is already completed
/// - `DatabaseError` - If the database operation fails
#[post("/work-sessions/{id}/pause")]
pub async fn pause_work_session(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let session_id = path.into_inner();

    let session =
        WorkSessionsRepo::find_by_id_for_user(&state.pool, auth_user.user_id, session_id)
            .await?
            .ok_or(ApiError::NotFound("Work session not found".to_string()))?;

    if session.end_time.is_some() {
        return Err(ApiError::ValidationError(
            "Cannot pause a completed work session".to_string(),
        ));
    }

    if !session.is_running {
        return Err(ApiError::ValidationError(
            "Work session is not currently running".to_string(),
        ));
    }

    let input = WorkSessionWriteInput {
        job_id: session.job_id,
        start_time: session.start_time,
        end_time: None,
        is_running: false,
        accumulated_paused_duration: session.accumulated_paused_duration,
        paused_at: Some(Utc::now()),
        time_reported: session.time_reported,
    };

    let work_session =
        WorkSessionsRepo::update_for_user(&state.pool, auth_user.user_id, session_id, &input)
            .await?
            .ok_or(ApiError::NotFound("Work session not found".to_string()))?;

    Ok(HttpResponse::Ok().json(WorkSessionResponse { work_session }))
}

/// Resumes a paused work session.
///
/// Adds the pause duration to `accumulated_paused_duration`, clears `paused_at`,
/// and sets `is_running` to true.
///
/// # Route
///
/// `POST /work-sessions/{id}/resume`
///
/// # Response Body ([`WorkSessionResponse`])
///
/// - `work_session` - The updated work session
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If the work session does not exist for the authenticated user
/// - `ValidationError` - If the session is not currently paused or is already completed
/// - `DatabaseError` - If the database operation fails
#[post("/work-sessions/{id}/resume")]
pub async fn resume_work_session(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let session_id = path.into_inner();

    let session =
        WorkSessionsRepo::find_by_id_for_user(&state.pool, auth_user.user_id, session_id)
            .await?
            .ok_or(ApiError::NotFound("Work session not found".to_string()))?;

    if session.end_time.is_some() {
        return Err(ApiError::ValidationError(
            "Cannot resume a completed work session".to_string(),
        ));
    }

    if session.is_running {
        return Err(ApiError::ValidationError(
            "Work session is already running".to_string(),
        ));
    }

    let paused_at = session
        .paused_at
        .ok_or(ApiError::ValidationError(
            "Work session is not currently paused".to_string(),
        ))?;

    let pause_duration = (Utc::now() - paused_at).num_seconds().max(0);

    let input = WorkSessionWriteInput {
        job_id: session.job_id,
        start_time: session.start_time,
        end_time: None,
        is_running: true,
        accumulated_paused_duration: session.accumulated_paused_duration + pause_duration,
        paused_at: None,
        time_reported: session.time_reported,
    };

    let work_session =
        WorkSessionsRepo::update_for_user(&state.pool, auth_user.user_id, session_id, &input)
            .await?
            .ok_or(ApiError::NotFound("Work session not found".to_string()))?;

    Ok(HttpResponse::Ok().json(WorkSessionResponse { work_session }))
}

/// Completes a work session.
///
/// If the session is currently paused, finalizes the pause duration before
/// setting `end_time` to the current time and `is_running` to false.
///
/// # Route
///
/// `POST /work-sessions/{id}/complete`
///
/// # Response Body ([`WorkSessionResponse`])
///
/// - `work_session` - The updated work session
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If the work session does not exist for the authenticated user
/// - `ValidationError` - If the session is already completed
/// - `DatabaseError` - If the database operation fails
#[post("/work-sessions/{id}/complete")]
pub async fn complete_work_session(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    path: web::Path<Uuid>,
) -> ApiResult<HttpResponse> {
    let session_id = path.into_inner();

    let session =
        WorkSessionsRepo::find_by_id_for_user(&state.pool, auth_user.user_id, session_id)
            .await?
            .ok_or(ApiError::NotFound("Work session not found".to_string()))?;

    if session.end_time.is_some() {
        return Err(ApiError::ValidationError(
            "Work session is already completed".to_string(),
        ));
    }

    let now = Utc::now();

    let accumulated = if let Some(paused_at) = session.paused_at {
        let pause_duration = (now - paused_at).num_seconds().max(0);
        session.accumulated_paused_duration + pause_duration
    } else {
        session.accumulated_paused_duration
    };

    let input = WorkSessionWriteInput {
        job_id: session.job_id,
        start_time: session.start_time,
        end_time: Some(now),
        is_running: false,
        accumulated_paused_duration: accumulated,
        paused_at: None,
        time_reported: session.time_reported,
    };

    let work_session =
        WorkSessionsRepo::update_for_user(&state.pool, auth_user.user_id, session_id, &input)
            .await?
            .ok_or(ApiError::NotFound("Work session not found".to_string()))?;

    Ok(HttpResponse::Ok().json(WorkSessionResponse { work_session }))
}

/// Retrieves the active work session for the authenticated user.
///
/// An active session is one with no `end_time` — either currently running or
/// paused.
///
/// # Route
///
/// `GET /work-sessions/active`
///
/// # Response Body ([`WorkSessionResponse`])
///
/// - `work_session` - The active work session
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `NotFound` - If no active work session exists
/// - `DatabaseError` - If the database operation fails
#[get("/work-sessions/active")]
pub async fn get_active_work_session(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> ApiResult<HttpResponse> {
    let work_session = WorkSessionsRepo::find_active_for_user(&state.pool, auth_user.user_id)
        .await?
        .ok_or(ApiError::NotFound(
            "No active work session found".to_string(),
        ))?;

    Ok(HttpResponse::Ok().json(WorkSessionResponse { work_session }))
}

#[cfg(test)]
mod tests {
    //! Handler-level tests for work session routes.
    //!
    //! Covered behavior:
    //! - Auth guard rejects unauthenticated work session requests.
    //! - Invalid UUID path parameters return a client error.
    //! - Missing request body returns a client error.

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

    use super::{get_active_work_session, pause_work_session, start_work_session};

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
    // Verifies unauthenticated start requests are rejected by the auth extractor.
    async fn start_work_session_without_access_cookie_returns_unauthorized() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .service(start_work_session),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/work-sessions/start")
            .set_json(serde_json::json!({ "job_id": Uuid::new_v4() }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies unauthenticated get-active requests are rejected by the auth extractor.
    async fn get_active_work_session_without_access_cookie_returns_unauthorized() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .service(get_active_work_session),
        )
        .await;

        let request = test::TestRequest::get()
            .uri("/work-sessions/active")
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies malformed UUID path segments fail route matching.
    async fn pause_work_session_with_invalid_uuid_path_returns_not_found() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "ws-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(pause_work_session),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/work-sessions/not-a-uuid/pause")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    // Verifies missing request body returns a client error for start endpoint.
    async fn start_work_session_with_empty_body_returns_bad_request() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "ws-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(start_work_session),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/work-sessions/start")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .insert_header(("Content-Type", "application/json"))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
