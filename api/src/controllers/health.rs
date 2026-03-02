use axum::Json;
use gig_log_common::models::health::HealthCheckResponse;

pub struct HeathController;

impl HeathController {
    pub async fn check_health() -> Json<HealthCheckResponse> {
        let response = HealthCheckResponse {
            status: "OK".to_string(),
        };

        Json::from(response)
    }
}
