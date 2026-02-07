//! HTTP handler for the health check endpoint.
//!
//! This module contains the handler function for the health check endpoint
//! used by load balancers and monitoring systems to verify API availability.

use actix_web::{HttpResponse, get};
use serde_json::json;

/// Returns the current health status of the API.
///
/// A simple endpoint that returns a 200 OK response when the API is running.
/// Used by load balancers, container orchestrators, and monitoring systems.
///
/// # Route
///
/// `GET /health`
///
/// # Response Body
///
/// ```json
/// {
///     "status": "ok"
/// }
/// ```
#[get("/health")]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}
