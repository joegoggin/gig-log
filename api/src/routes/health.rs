use axum::{routing::get, Router};

use crate::controllers::health::HeathController;
use crate::routes::app::AppState;

pub struct HealthRouter;

impl HealthRouter {
    pub fn new() -> Router<AppState> {
        Router::new().route("/", get(HeathController::check_health))
    }
}
