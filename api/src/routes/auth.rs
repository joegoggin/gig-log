//! Authentication and account management route definitions.
//!
//! This module defines the [`AuthRouter`], which maps HTTP endpoints to
//! [`AuthController`] handler
//! methods for sign-up, login, password management, and email change flows.

use axum::{
    Router,
    routing::{get, post},
};

use crate::{controllers::auth::AuthController, routes::app::AppState};

/// Router for authentication and account management endpoints.
pub struct AuthRouter;

impl AuthRouter {
    /// Creates a [`Router`] with all authentication routes.
    ///
    /// Registers the following endpoints under the `/auth` prefix:
    ///
    /// - `POST /sign-up` — Register a new user account.
    /// - `POST /confirm-email` — Confirm an email address.
    /// - `POST /log-in` — Authenticate and obtain tokens.
    /// - `POST /log-out` — Revoke the current session.
    /// - `POST /refresh` — Refresh an access token.
    /// - `GET /me` — Retrieve the authenticated user's profile.
    /// - `POST /forgot-password` — Request a password reset code.
    /// - `POST /verify-forgot-password` — Verify a password reset code.
    /// - `POST /set-password` — Set a new password after reset.
    /// - `POST /request-change-password` — Request a password change code.
    /// - `POST /change-password` — Change the current password.
    /// - `POST /request-email-change` — Request an email change.
    /// - `POST /confirm-email-change` — Confirm an email change.
    ///
    /// # Returns
    ///
    /// A [`Router<AppState>`] with all authentication routes registered.
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
            .route(
                "/request-change-password",
                post(AuthController::request_change_password_code),
            )
            .route("/change-password", post(AuthController::change_password))
            .route(
                "/request-email-change",
                post(AuthController::request_email_change),
            )
            .route(
                "/confirm-email-change",
                post(AuthController::confirm_email_change),
            )
    }
}
