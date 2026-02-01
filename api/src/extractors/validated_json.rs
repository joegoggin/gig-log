use actix_web::{FromRequest, HttpRequest, HttpResponse, dev::Payload, web};
use futures::future::LocalBoxFuture;
use serde::{Serialize, de::DeserializeOwned};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ValidationErrorResponse {
    pub errors: Vec<FieldError>,
}

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

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
