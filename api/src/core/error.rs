use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use gig_log_common::models::error::{ApiError, ValidationError};

pub type ApiResult<T> = Result<T, ApiErrorResponse>;

#[derive(Debug)]
pub enum ApiErrorResponse {
    NotFound(String),
    BadRequest(String),
    Validation(Vec<ValidationError>),
    InternalServerError(String),
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let (status, message, errors) = match self {
            ApiErrorResponse::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            ApiErrorResponse::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, None),
            ApiErrorResponse::Validation(errs) => (
                StatusCode::BAD_REQUEST,
                "Validation Error".to_string(),
                Some(errs),
            ),
            ApiErrorResponse::InternalServerError(msg) => {
                println!("Error: {:#?}", msg);

                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong".to_string(),
                    None,
                )
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

impl From<JsonRejection> for ApiErrorResponse {
    fn from(err: JsonRejection) -> Self {
        ApiErrorResponse::BadRequest(err.body_text())
    }
}
