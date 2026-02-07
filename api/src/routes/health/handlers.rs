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

#[cfg(test)]
mod tests {
    use actix_web::{App, http::StatusCode, test};
    use serde_json::json;

    use super::health_check;

    #[actix_web::test]
    // Verifies the health endpoint returns the expected OK payload.
    async fn health_check_returns_ok_payload() {
        let app = test::init_service(App::new().service(health_check)).await;
        let request = test::TestRequest::get().uri("/health").to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(body, json!({ "status": "ok" }));
    }
}
