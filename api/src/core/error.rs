//! API error types and Axum response conversions.
//!
//! This module defines [`ApiErrorResponse`], the canonical error enum returned
//! by route handlers. Each variant maps to an HTTP status code and is
//! automatically serialized into a JSON [`ApiError`] body via its
//! [`IntoResponse`] implementation.

use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use gig_log_common::models::error::{ApiError, ValidationError};
use log::{error, warn};

/// Convenience alias for handler return types that may fail with an [`ApiErrorResponse`].
pub type ApiResult<T> = Result<T, ApiErrorResponse>;

/// An error that can be returned from an API route handler.
///
/// Each variant carries a human-readable message and maps to a specific HTTP
/// status code. The [`IntoResponse`] implementation logs the
/// error and serializes it as a JSON [`ApiError`].
#[derive(Debug)]
pub enum ApiErrorResponse {
    /// Resource not found. Returns HTTP `404 Not Found`.
    NotFound(String),
    /// Client sent an invalid request. Returns HTTP `400 Bad Request`.
    BadRequest(String),
    /// One or more fields failed validation. Returns HTTP `400 Bad Request`.
    Validation(Vec<ValidationError>),
    /// Unexpected server-side failure. Returns HTTP `500 Internal Server Error`.
    ///
    /// The original message is logged but not exposed to the client; the
    /// response body always reads "Something went wrong".
    InternalServerError(String),
    /// Missing or invalid authentication credentials. Returns HTTP `401 Unauthorized`.
    Unauthorized(String),
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let (status, message, errors) = match self {
            ApiErrorResponse::NotFound(msg) => {
                warn!("NotFound: {}", msg);
                (StatusCode::NOT_FOUND, msg, None)
            }
            ApiErrorResponse::BadRequest(msg) => {
                warn!("BadRequest: {}", msg);
                (StatusCode::BAD_REQUEST, msg, None)
            }
            ApiErrorResponse::Validation(errs) => {
                warn!("Validation error: {:?}", errs);

                (
                    StatusCode::BAD_REQUEST,
                    "Validation Error".to_string(),
                    Some(errs),
                )
            }
            ApiErrorResponse::InternalServerError(msg) => {
                error!("InternalServerError: {}", msg);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                    None,
                )
            }
            ApiErrorResponse::Unauthorized(msg) => {
                warn!("Unauthorized: {}", msg);
                (StatusCode::UNAUTHORIZED, msg, None)
            }
        };

        let body = ApiError {
            status_code: status.as_u16(),
            message,
            errors,
        };

        (status, Json(body)).into_response()
    }
}

/// Converts a database error into an [`ApiErrorResponse`].
///
/// [`sqlx::Error::RowNotFound`] becomes [`NotFound`](ApiErrorResponse::NotFound);
/// all other database errors become
/// [`InternalServerError`](ApiErrorResponse::InternalServerError).
impl From<sqlx::Error> for ApiErrorResponse {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                ApiErrorResponse::NotFound("Resource not found".to_string())
            }
            error => ApiErrorResponse::InternalServerError(error.to_string()),
        }
    }
}

/// Converts validator field errors into an [`ApiErrorResponse::Validation`] variant.
impl From<validator::ValidationErrors> for ApiErrorResponse {
    fn from(errs: validator::ValidationErrors) -> Self {
        let validation_errors = errs
            .field_errors()
            .into_iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| ValidationError {
                    field: Some(field.to_string()),
                    message: e.message.clone().map(|m| m.to_string()).unwrap_or_default(),
                })
            })
            .collect();

        ApiErrorResponse::Validation(validation_errors)
    }
}

/// Converts an Axum JSON extraction rejection into an [`ApiErrorResponse::BadRequest`].
impl From<JsonRejection> for ApiErrorResponse {
    fn from(err: JsonRejection) -> Self {
        ApiErrorResponse::BadRequest(err.body_text())
    }
}
