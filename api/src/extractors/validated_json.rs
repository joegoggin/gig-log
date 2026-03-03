use axum::{
    extract::rejection::JsonRejection,
    extract::{FromRequest, Json, Request},
};
use validator::Validate;

use crate::core::error::ApiErrorResponse;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T> ValidatedJson<T> {
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

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(ApiErrorResponse::from)?;

        value.validate().map_err(ApiErrorResponse::from)?;

        Ok(Self(value))
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
        #[serde(default)]
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
