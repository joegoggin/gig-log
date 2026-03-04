use axum::{
    Router,
    routing::{get, post},
};

use crate::{controllers::auth::AuthController, routes::app::AppState};

pub struct AuthRouter;

impl AuthRouter {
    pub fn new() -> Router<AppState> {
        Router::new()
            .route("/sign-up", post(AuthController::sign_up))
            .route("/confirm-email", post(AuthController::confirm_email))
            .route("/log-in", post(AuthController::log_in))
            .route("/log-out", post(AuthController::log_out))
            .route("/refresh", post(AuthController::refresh))
            .route("/me", get(AuthController::me))
            .route("/forgot-password", post(AuthController::forgot_password))
            .route(
                "/verify-forgot-password",
                post(AuthController::verify_forgot_password),
            )
            .route("/set-password", post(AuthController::set_password))
            .route("/change-password", post(AuthController::change_password))
            .route(
                "/change-password/request-code",
                post(AuthController::request_change_password_code),
            )
    }
}
