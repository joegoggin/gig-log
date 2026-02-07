//! JSON extractor with request payload validation.
//!
//! This module extends `actix-web` JSON extraction by validating deserialized
//! payloads with the `validator` crate before control reaches route handlers.
//! Validation failures are converted into a consistent `400 Bad Request`
//! response body that lists field-level errors.

use actix_web::{FromRequest, HttpRequest, HttpResponse, dev::Payload, web};
use futures::future::LocalBoxFuture;
use serde::{Serialize, de::DeserializeOwned};
use validator::{Validate, ValidationErrors};

/// A single validation error associated with a request field.
#[derive(Debug, Serialize)]
pub struct FieldError {
    /// Name of the invalid field.
    ///
    /// For schema-level validation errors (`__all__`), this is set to the
    /// validator error code.
    pub field: String,
    /// Human-readable validation message for the field.
    pub message: String,
}

/// JSON response body returned when payload validation fails.
#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    /// Collection of field-level validation failures.
    pub errors: Vec<FieldError>,
}

/// Extracts JSON into `T` and validates it with [`Validate`].
///
/// This extractor can be used in handler signatures in place of
/// `web::Json<T>` when the payload type implements `validator::Validate`.
/// If validation fails, the request is rejected with `400 Bad Request`.
#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    /// Consumes the extractor and returns the validated payload.
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Internal rejection type used to return validation errors from extraction.
#[derive(Debug)]
pub struct ValidationRejection(pub ValidationErrorResponse);

impl std::fmt::Display for ValidationRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Validation failed")
    }
}

impl actix_web::ResponseError for ValidationRejection {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::BadRequest().json(&self.0)
    }
}

/// Converts validator errors into the API's standard validation response shape.
///
/// Nested struct/list validation errors are flattened into a single `errors`
/// vector so clients can display all violations from one response.
///
/// # Arguments
///
/// - `errors` - Validation errors produced by `validator::Validate::validate`.
fn convert_validation_errors(errors: ValidationErrors) -> ValidationErrorResponse {
    let mut field_errors = Vec::new();

    for (field, errs) in errors.errors() {
        match errs {
            validator::ValidationErrorsKind::Field(field_errs) => {
                for error in field_errs {
                    let message = error
                        .message
                        .as_ref()
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| format!("{} is invalid", field));

                    // For schema-level errors (__all__), use empty field or the error code
                    let field_name = if field == "__all__" {
                        error.code.to_string()
                    } else {
                        field.to_string()
                    };

                    field_errors.push(FieldError {
                        field: field_name,
                        message,
                    });
                }
            }
            validator::ValidationErrorsKind::Struct(nested) => {
                let nested_response = convert_validation_errors(*nested.clone());
                field_errors.extend(nested_response.errors);
            }
            validator::ValidationErrorsKind::List(list) => {
                for (_, err) in list {
                    let nested = convert_validation_errors(*err.clone());
                    field_errors.extend(nested.errors);
                }
            }
        }
    }

    ValidationErrorResponse {
        errors: field_errors,
    }
}

impl<T> FromRequest for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + 'static,
{
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let json_fut = web::Json::<T>::from_request(req, payload);

        Box::pin(async move {
            let json = json_fut.await?;
            let inner = json.into_inner();

            if let Err(validation_errors) = inner.validate() {
                let response = convert_validation_errors(validation_errors);
                return Err(ValidationRejection(response).into());
            }

            Ok(ValidatedJson(inner))
        })
    }
}

#[cfg(test)]
mod tests {
    use actix_web::{App, HttpResponse, Responder, http::StatusCode, post, test};
    use serde::Deserialize;
    use serde_json::json;
    use validator::Validate;

    use super::ValidatedJson;

    #[derive(Debug, Deserialize, Validate)]
    struct TestPayload {
        #[validate(length(min = 1, message = "Name is required"))]
        name: String,
    }

    #[post("/validated")]
    async fn validated(body: ValidatedJson<TestPayload>) -> impl Responder {
        let payload = body.into_inner();
        HttpResponse::Ok().body(payload.name)
    }

    #[actix_web::test]
    // Verifies invalid payloads are rejected with the extractor error shape.
    async fn returns_bad_request_with_validation_errors_when_payload_is_invalid() {
        let app = test::init_service(App::new().service(validated)).await;
        let request = test::TestRequest::post()
            .uri("/validated")
            .set_json(json!({ "name": "" }))
            .to_request();

        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: serde_json::Value = test::read_body_json(response).await;
        let errors = body
            .get("errors")
            .and_then(|value| value.as_array())
            .expect("validation response should include an errors array");

        assert!(
            errors
                .iter()
                .any(|error| error.get("field") == Some(&json!("name")))
        );
    }

    #[actix_web::test]
    // Verifies valid payloads pass extractor validation and reach the handler.
    async fn allows_request_when_payload_is_valid() {
        let app = test::init_service(App::new().service(validated)).await;
        let request = test::TestRequest::post()
            .uri("/validated")
            .set_json(json!({ "name": "GigLog" }))
            .to_request();

        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::OK);
    }
}
