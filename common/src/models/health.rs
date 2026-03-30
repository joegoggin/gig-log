use serde::{Deserialize, Serialize};

/// Response returned by the health check endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Current health status (e.g., "ok").
    pub status: String,
}
