use axum::{Router, routing::post};

use crate::{controllers::auth::AuthController, routes::app::AppState};

pub struct AuthRouter;

impl AuthRouter {
    pub fn new() -> Router<AppState> {
        Router::new()
            .route("/sign-up", post(AuthController::sign_up))
            .route("/confirm-email", post(AuthController::confirm_email))
    }
}
