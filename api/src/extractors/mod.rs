//! Request extractors for API handlers.
//!
//! This module provides custom `actix-web` extractors that deserialize
//! incoming request data and enforce validation before handler logic runs.
//!
//! # Exports
//!
//! - [`ValidatedJson`] - JSON body extractor that validates payloads with the
//!   `validator` crate and returns a standardized `400 Bad Request` response.

mod validated_json;

/// JSON body extractor that deserializes and validates request payloads.
pub use validated_json::ValidatedJson;
