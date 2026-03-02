use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use gig_log_common::models::error::{ApiError, ValidationError};

pub type ApiResult<T> = Result<T, ApiErrorResposne>;

#[derive(Debug)]
pub enum ApiErrorResposne {
    NotFound(String),
    BadRequest(String),
    Validation(Vec<ValidationError>),
    InternalServerError(String),
}

impl IntoResponse for ApiErrorResposne {
    fn into_response(self) -> Response {
        let (status, message, errors) = match self {
            ApiErrorResposne::NotFound(msg) => (StatusCode::NOT_FOUND, msg, None),
            ApiErrorResposne::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, None),
            ApiErrorResposne::Validation(errs) => (
                StatusCode::BAD_REQUEST,
                "Validation failed".to_string(),
                Some(errs),
            ),
            ApiErrorResposne::InternalServerError(msg) => {
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

impl From<sqlx::Error> for ApiErrorResposne {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => {
                ApiErrorResposne::NotFound("Resource not found".to_string())
            }
            error => ApiErrorResposne::InternalServerError(error.to_string()),
        }
    }
}

impl From<validator::ValidationErrors> for ApiErrorResposne {
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

        ApiErrorResposne::Validation(validation_errors)
    }
}
