//! Health check route definitions.
//!
//! This module defines the [`HealthRouter`], which exposes a single endpoint
//! for verifying that the API is running.

use axum::{routing::get, Router};

use crate::controllers::health::HeathController;
use crate::routes::app::AppState;

/// Router for health check endpoints.
pub struct HealthRouter;

impl HealthRouter {
    /// Creates a [`Router`] with the health check route.
    ///
    /// Registers `GET /` mapped to
    /// [`HeathController::check_health`](crate::controllers::health::HeathController::check_health).
    ///
    /// # Returns
    ///
    /// A [`Router<AppState>`] with the health check route registered.
    pub fn new() -> Router<AppState> {
        Router::new().route("/", get(HeathController::check_health))
    }
}
