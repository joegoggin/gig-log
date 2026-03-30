//! Validated JSON extractor for Axum request handlers.
//!
//! This module provides [`ValidatedJson<T>`], a custom Axum extractor that
//! combines JSON deserialization with automatic validation using the
//! [`validator`] crate. When extraction fails, structured error responses
//! are returned to clients as [`ApiErrorResponse`] variants.

use axum::{
    extract::rejection::JsonRejection,
    extract::{FromRequest, Json, Request},
};
use gig_log_common::models::error::ValidationError;
use validator::Validate;

use crate::core::error::ApiErrorResponse;

/// Axum extractor that deserializes JSON and validates the result.
///
/// Wraps Axum's [`Json`] extractor with an additional validation step using the
/// [`Validate`] trait. If deserialization or validation fails, a structured
/// [`ApiErrorResponse`] is returned to the client.
#[derive(Debug)]
pub struct ValidatedJson<T>(
    /// The validated inner value.
    pub T,
);

impl<T> ValidatedJson<T> {
    /// Consumes the extractor and returns the inner validated value.
    ///
    /// # Returns
    ///
    /// The inner `T` that was deserialized and validated.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: Validate + Send,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ApiErrorResponse;

    /// Extracts and validates a JSON payload from an incoming request.
    ///
    /// Deserializes the request body as JSON using Axum's [`Json`] extractor,
    /// then runs [`Validate::validate`] on the result. Missing-field errors
    /// from serde are converted into structured [`ApiErrorResponse::Validation`]
    /// responses with human-readable field names.
    ///
    /// # Arguments
    ///
    /// * `req` — The incoming HTTP request.
    /// * `state` — The Axum application state.
    ///
    /// # Returns
    ///
    /// A [`ValidatedJson<T>`] containing the deserialized and validated value.
    ///
    /// # Errors
    ///
    /// Returns [`ApiErrorResponse::Validation`] if a required field is missing
    /// or validation constraints are violated.
    /// Returns [`ApiErrorResponse::BadRequest`] if JSON deserialization fails
    /// for other reasons (e.g., type mismatch).
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| match err {
                JsonRejection::JsonDataError(error) => {
                    let message = error.body_text();

                    map_missing_field_error(&message)
                        .unwrap_or_else(|| ApiErrorResponse::BadRequest(message))
                }
                error => ApiErrorResponse::from(error),
            })?;

        value.validate().map_err(ApiErrorResponse::from)?;

        Ok(Self(value))
    }
}

/// Maps a serde missing-field error message into a structured validation error.
///
/// # Arguments
///
/// * `message` — The error message from serde's JSON deserialization.
///
/// # Returns
///
/// An [`ApiErrorResponse::Validation`] if the message contains a missing-field
/// error, or [`None`] if the message does not match the expected pattern.
fn map_missing_field_error(message: &str) -> Option<ApiErrorResponse> {
    let field = extract_missing_field_name(message)?;

    Some(ApiErrorResponse::Validation(vec![ValidationError {
        field: Some(field.to_string()),
        message: format!("{} is required", format_field_name(field)),
    }]))
}

/// Extracts the field name from a serde missing-field error message.
///
/// Parses messages matching the pattern ``missing field `field_name` `` and
/// returns the field name.
///
/// # Arguments
///
/// * `message` — The error message from serde's JSON deserialization.
///
/// # Returns
///
/// The extracted field name, or [`None`] if the message does not match
/// the expected pattern.
fn extract_missing_field_name(message: &str) -> Option<&str> {
    let prefix = "missing field `";
    let start = message.find(prefix)? + prefix.len();
    let remainder = &message[start..];
    let end = remainder.find('`')?;

    Some(&remainder[..end])
}

/// Formats a snake_case field name into a human-readable, capitalized string.
///
/// Replaces underscores with spaces and capitalizes the first character.
///
/// # Arguments
///
/// * `field` — The snake_case field name.
///
/// # Returns
///
/// A [`String`] with underscores replaced by spaces and the first letter
/// capitalized (e.g., `"first_name"` becomes `"First name"`).
fn format_field_name(field: &str) -> String {
    let normalized = field.replace('_', " ");
    let mut chars = normalized.chars();

    match chars.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), chars.as_str()),
        None => normalized,
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::FromRequest,
        http::{Request, StatusCode, header::CONTENT_TYPE},
        response::IntoResponse,
    };
    use serde::Deserialize;
    use validator::Validate;

    use crate::core::error::ApiErrorResponse;

    use super::ValidatedJson;

    #[derive(Debug, Deserialize, Validate)]
    struct TestPayload {
        #[validate(length(min = 1, message = "Name is required"))]
        name: String,
    }

    #[tokio::test]
    async fn rejects_payload_when_validation_fails() {
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name":""}"#))
            .expect("request should build");

        let err = ValidatedJson::<TestPayload>::from_request(req, &())
            .await
            .expect_err("extractor should reject invalid payload");

        match &err {
            ApiErrorResponse::Validation(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field.as_deref(), Some("name"));
            }
            _ => panic!("expected validation error"),
        }

        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_payload_when_required_field_is_missing() {
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{}"#))
            .expect("request should build");

        let err = ValidatedJson::<TestPayload>::from_request(req, &())
            .await
            .expect_err("extractor should reject missing required fields");

        match &err {
            ApiErrorResponse::Validation(errors) => {
                assert_eq!(errors.len(), 1);
                assert_eq!(errors[0].field.as_deref(), Some("name"));
            }
            _ => panic!("expected validation error"),
        }

        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn rejects_payload_when_json_is_invalid() {
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name":123}"#))
            .expect("request should build");

        let err = ValidatedJson::<TestPayload>::from_request(req, &())
            .await
            .expect_err("extractor should reject invalid json");

        match &err {
            ApiErrorResponse::BadRequest(message) => {
                assert!(message.contains("Failed to deserialize"));
            }
            _ => panic!("expected bad request error"),
        }

        assert_eq!(err.into_response().status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn allows_payload_when_json_and_validation_are_valid() {
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"name":"GigLog"}"#))
            .expect("request should build");

        let extracted = ValidatedJson::<TestPayload>::from_request(req, &())
            .await
            .expect("extractor should accept valid payload");

        let body = extracted.into_inner();
        assert_eq!(body.name, "GigLog");
    }
}
