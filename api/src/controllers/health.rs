//! Health check endpoint.
//!
//! Provides [`HeathController`] with a single handler that reports the
//! current API status.

use axum::Json;
use gig_log_common::models::health::HealthCheckResponse;

/// Handler for the health check endpoint.
pub struct HeathController;

impl HeathController {
    /// Returns the current health status of the API.
    ///
    /// Mapped to `GET /health`.
    ///
    /// # Returns
    ///
    /// A [`Json<HealthCheckResponse>`] with a status of `"OK"`.
    pub async fn check_health() -> Json<HealthCheckResponse> {
        let response = HealthCheckResponse {
            status: "OK".to_string(),
        };

        Json::from(response)
    }
}
