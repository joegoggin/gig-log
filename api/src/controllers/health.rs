use axum::{Json, extract::State};
use gig_log_common::models::health::HealthCheckResponse;

use crate::routes::app::AppState;

pub struct HeathController;

impl HeathController {
    pub async fn check_health(state: State<AppState>) -> Json<HealthCheckResponse> {
        let response = HealthCheckResponse {
            status: "OK".to_string(),
        };

        Json::from(response)
    }
}
