//! Error types used by frontend API client wrappers.

use gig_log_common::models::error::ApiError;

/// Represents errors returned by frontend API client calls.
#[derive(Debug, Clone)]
pub enum ClientError {
    /// Wraps a structured API error response.
    Api(ApiError),
    /// Wraps a client-side network or serialization failure message.
    Network(String),
}

impl std::fmt::Display for ClientError {
    /// Formats the client error for user-facing display.
    ///
    /// # Arguments
    ///
    /// * `f` — Formatter receiving the rendered message.
    ///
    /// # Returns
    ///
    /// A [`std::fmt::Result`] indicating formatting success.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Api(err) => write!(f, "{}", err.message),
            ClientError::Network(msg) => write!(f, "Network error: {}", msg),
        }
    }
}
