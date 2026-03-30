//! Custom Axum extractors for the GigLog API.
//!
//! This module provides request extractors that extend Axum's built-in
//! extraction capabilities with additional functionality such as input
//! validation.
//!
//! # Modules
//!
//! - `validated_json` — JSON extractor with automatic validation.

mod validated_json;

pub use validated_json::ValidatedJson;
