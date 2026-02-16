//! Custom logging utilities and HTTP request/response logging middleware.
//!
//! This module provides:
//! - A custom `log` backend with colorized console output
//! - Actix middleware that logs HTTP requests and responses in a readable format
//! - Redaction helpers to avoid leaking sensitive headers and JSON fields

use std::{env, time::Instant};

use actix_web::{
    Error, HttpMessage,
    body::{BoxBody, MessageBody},
    dev::{Payload, ServiceRequest, ServiceResponse},
    http::{
        Method, StatusCode,
        header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderMap},
    },
    middleware::Next,
    web,
};
use colorized::{Colors, colorize_print, colorize_println};
use futures::StreamExt;
use log::{Level, LevelFilter, Log, Metadata, Record, set_logger, set_max_level};
use serde_json::{Value, from_slice};
use uuid::Uuid;

/// Runtime options that control HTTP request/response logging behavior.
#[derive(Debug, Clone)]
pub struct HttpLoggingConfig {
    /// Whether HTTP body logging is enabled.
    pub body_enabled: bool,
    /// Maximum number of bytes eligible for body logging.
    pub max_body_bytes: usize,
}

impl Default for HttpLoggingConfig {
    fn default() -> Self {
        Self {
            body_enabled: true,
            max_body_bytes: 16 * 1024,
        }
    }
}

/// Custom application logger.
///
/// This logger implements the `log` crate backend and exposes helpers used
/// by server startup and HTTP middleware for consistent formatting.
pub struct Logger;

static LOGGER: Logger = Logger;

impl Logger {
    /// Initializes global logging using a level string (`info`, `debug`, etc.).
    ///
    /// If a global logger was already set (e.g. by another test), initialization
    /// is skipped and only the max level is updated.
    ///
    /// # Arguments
    ///
    /// - `log_level` - Case-insensitive log level string.
    pub fn setup_logging(log_level: &str) {
        let _ = set_logger(&LOGGER);
        set_max_level(Self::parse_level_filter(log_level));
    }

    /// Initializes logging by reading `LOG_LEVEL` from environment.
    ///
    /// This loads `.env` (if present) before reading the variable and defaults
    /// to `info` when unset or invalid.
    pub fn setup_logging_from_env() {
        dotenvy::dotenv().ok();

        let level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        Self::setup_logging(&level);
    }

    /// Logs a startup/status message in green.
    ///
    /// # Arguments
    ///
    /// - `message` - Message to display.
    pub fn log_success(message: &str) {
        let hashtags = Self::get_hashtags(6);
        let message = format!("\n{} {} {}\n", hashtags, message, hashtags);
        colorize_println(message, Colors::GreenFg);
    }

    /// Logs a startup/status message in blue.
    ///
    /// # Arguments
    ///
    /// - `message` - Message to display.
    pub fn log_message(message: &str) {
        let hashtags = Self::get_hashtags(6);
        let message = format!("\n{} {} {}\n", hashtags, message, hashtags);
        colorize_println(message, Colors::BlueFg);
    }

    /// Actix middleware that logs requests and responses.
    ///
    /// The middleware logs:
    /// - Request ID
    /// - Method and path
    /// - Sanitized headers
    /// - Optional redacted JSON request/response bodies
    /// - Response status and duration
    ///
    /// Body logging is controlled by [`HttpLoggingConfig`] from app data.
    ///
    /// # Errors
    ///
    /// Returns downstream handler/middleware errors from `next.call(req)`.
    /// Response body capture failures are treated as non-fatal and only
    /// disable body logging for that response.
    pub async fn log_request_and_response<B>(
        mut req: ServiceRequest,
        next: Next<B>,
    ) -> Result<ServiceResponse<BoxBody>, Error>
    where
        B: MessageBody + 'static,
    {
        if !log::log_enabled!(Level::Info) {
            return next
                .call(req)
                .await
                .map(ServiceResponse::map_into_boxed_body);
        }

        let request_id = Uuid::new_v4();
        let method = req.method().clone();
        let path = req.path().to_string();
        let headers = req.headers().clone();
        let config = req
            .app_data::<web::Data<HttpLoggingConfig>>()
            .map(|cfg| cfg.get_ref().clone())
            .unwrap_or_default();

        let request_body = if Self::should_attempt_body_logging(&method, &headers, &config) {
            Self::capture_request_body(&mut req).await
        } else {
            None
        };

        Self::log_request(request_id, &method, &path, &headers, request_body.clone());

        let started_at = Instant::now();
        let response_result = next.call(req).await;
        let mut response = match response_result {
            Ok(response) => response.map_into_boxed_body(),
            Err(error) => {
                let duration_ms = started_at.elapsed().as_millis();
                let status = error.as_response_error().status_code();
                Self::log_response(request_id, status, duration_ms, None);
                return Err(error);
            }
        };
        let duration_ms = started_at.elapsed().as_millis();
        let status = response.status();

        let response_headers = response.headers().clone();
        let response_body =
            if Self::should_attempt_response_body_logging(&response_headers, &config) {
                let (req_head, http_response) = response.into_parts();
                let (http_response_head, response_body_stream) = http_response.into_parts();

                match response_body_stream.try_into_bytes() {
                    Ok(bytes) => {
                        let logged_body = if bytes.len() <= config.max_body_bytes {
                            Self::parse_redacted_json(&bytes)
                        } else {
                            None
                        };
                        let rebuilt = http_response_head.set_body(bytes).map_into_boxed_body();
                        response = ServiceResponse::new(req_head, rebuilt);

                        logged_body
                    }
                    Err(response_body_stream) => {
                        log::debug!(
                            "Skipping response body logging for request {} (non-bufferable body)",
                            request_id
                        );
                        let rebuilt = http_response_head
                            .set_body(response_body_stream)
                            .map_into_boxed_body();
                        response = ServiceResponse::new(req_head, rebuilt);

                        None
                    }
                }
            } else {
                None
            };

        Self::log_response(request_id, status, duration_ms, response_body);

        Ok(response)
    }

    fn parse_level_filter(log_level: &str) -> LevelFilter {
        match log_level.trim().to_ascii_lowercase().as_str() {
            "off" => LevelFilter::Off,
            "error" => LevelFilter::Error,
            "warn" => LevelFilter::Warn,
            "info" => LevelFilter::Info,
            "debug" => LevelFilter::Debug,
            "trace" => LevelFilter::Trace,
            _ => LevelFilter::Info,
        }
    }

    fn should_attempt_body_logging(
        method: &Method,
        headers: &HeaderMap,
        config: &HttpLoggingConfig,
    ) -> bool {
        if !config.body_enabled {
            return false;
        }

        if matches!(method, &Method::GET | &Method::DELETE | &Method::HEAD) {
            return false;
        }

        if !Self::is_json_content_type(headers) {
            return false;
        }

        match Self::content_length(headers) {
            Some(length) => length <= config.max_body_bytes,
            None => false,
        }
    }

    fn should_attempt_response_body_logging(
        headers: &HeaderMap,
        config: &HttpLoggingConfig,
    ) -> bool {
        if !config.body_enabled {
            return false;
        }

        if !Self::is_json_content_type(headers) {
            return false;
        }

        match Self::content_length(headers) {
            Some(length) => length <= config.max_body_bytes,
            None => true,
        }
    }

    async fn capture_request_body(req: &mut ServiceRequest) -> Option<Value> {
        let mut payload = req.take_payload();
        let mut bytes = web::BytesMut::new();

        while let Some(chunk_result) = payload.next().await {
            let chunk = match chunk_result {
                Ok(chunk) => chunk,
                Err(error) => {
                    log::error!("Failed reading request body for logging: {}", error);
                    return None;
                }
            };

            bytes.extend_from_slice(&chunk);
        }

        req.set_payload(Payload::from(bytes.clone().freeze()));

        Self::parse_redacted_json(&bytes)
    }

    fn parse_redacted_json(bytes: &[u8]) -> Option<Value> {
        if bytes.is_empty() {
            return None;
        }

        let mut value = from_slice::<Value>(bytes).ok()?;
        Self::redact_json_value(&mut value);

        Some(value)
    }

    fn redact_json_value(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for (key, value) in map.iter_mut() {
                    if Self::is_sensitive_json_key(key) {
                        *value = Value::String("(hidden)".to_string());
                    } else {
                        Self::redact_json_value(value);
                    }
                }
            }
            Value::Array(items) => {
                for item in items {
                    Self::redact_json_value(item);
                }
            }
            _ => {}
        }
    }

    fn is_sensitive_json_key(key: &str) -> bool {
        let key = key.to_ascii_lowercase();

        matches!(
            key.as_str(),
            "password"
                | "confirm"
                | "token"
                | "access_token"
                | "refresh_token"
                | "auth_code"
                | "secret"
                | "api_key"
        ) || key.contains("token")
            || key.contains("secret")
            || key.contains("password")
    }

    fn is_json_content_type(headers: &HeaderMap) -> bool {
        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_ascii_lowercase());

        match content_type {
            Some(content_type) => {
                content_type.contains("application/json") || content_type.contains("+json")
            }
            None => false,
        }
    }

    fn content_length(headers: &HeaderMap) -> Option<usize> {
        headers
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
    }

    fn sanitize_header_value(name: &str, value: &str) -> String {
        if Self::is_sensitive_header(name) {
            "(hidden)".to_string()
        } else {
            value.to_string()
        }
    }

    fn is_sensitive_header(name: &str) -> bool {
        let name = name.to_ascii_lowercase();

        matches!(
            name.as_str(),
            "authorization" | "cookie" | "set-cookie" | "x-api-key" | "x-auth-token"
        ) || name.contains("token")
            || name.contains("secret")
    }

    fn get_hashtags(level: i8) -> String {
        let mut hashtags = String::new();

        for _ in 0..level {
            hashtags.push('#');
        }

        hashtags
    }

    fn get_spaces(level: i8) -> String {
        let mut spaces = String::new();

        for _ in 0..level {
            spaces.push_str("  ");
        }

        spaces
    }

    fn log_header(header: &str, hashtags: &str) {
        let header_string = format!("\n{} {} {}\n", hashtags, header, hashtags);
        colorize_println(header_string, Colors::MagentaFg);
    }

    fn log_h1(header: &str) {
        Self::log_header(header, &Self::get_hashtags(6));
    }

    fn log_h2(header: &str) {
        Self::log_header(header, &Self::get_hashtags(5));
    }

    fn log_json(json: Value, level: i8) {
        match json {
            Value::Array(values) => {
                Self::log_array(values, level);
            }
            json => {
                if let Some(json_object) = json.as_object() {
                    if level == 0 {
                        println!("{{");
                    }

                    for (key, value) in json_object {
                        print!("{}", Self::get_spaces(level + 1));
                        colorize_print(key, Colors::CyanFg);
                        print!(": ");

                        match value {
                            Value::Array(values) => Self::log_array(values.clone(), level + 1),
                            Value::String(value) => {
                                colorize_print(format!("\"{}\"", value), Colors::GreenFg);
                                println!(",");
                            }
                            Value::Object(value) => {
                                println!("{{");
                                Self::log_json(Value::Object(value.clone()), level + 2);
                                print!("{}", Self::get_spaces(level + 1));
                                println!("}},");
                            }
                            value => {
                                colorize_print(value.to_string(), Colors::MagentaFg);
                                println!(",");
                            }
                        }
                    }

                    if level == 0 {
                        println!("}}");
                    }
                }
            }
        }
    }

    fn log_array(values: Vec<Value>, level: i8) {
        if values.is_empty() {
            println!("[],");
            return;
        }

        if level > 0 {
            print!("\n{}", Self::get_spaces(level));
        }

        println!("[");

        for (index, value) in values.iter().enumerate() {
            print!("{}", Self::get_spaces(level + 1));
            println!("{{");
            Self::log_json(value.to_owned(), level + 2);
            print!("{}", Self::get_spaces(level + 1));

            if index == values.len() - 1 {
                println!("}}");
            } else {
                println!("}},");
            }
        }

        if level > 0 {
            print!("{}", Self::get_spaces(level));
        }

        if level == 0 {
            println!("]");
        } else {
            println!("],");
        }
    }

    fn log_headers(headers: &HeaderMap) {
        for (key, value) in headers {
            colorize_print(format!("{}: ", key), Colors::CyanFg);
            let value = value.to_str().unwrap_or("<non-utf8>");
            let sanitized = Self::sanitize_header_value(key.as_str(), value);
            println!("{}", sanitized);
        }
    }

    fn log_request(
        id: Uuid,
        method: &Method,
        route: &str,
        headers: &HeaderMap,
        req_body: Option<Value>,
    ) {
        Self::log_h1("Request");

        colorize_print("Request ID: ", Colors::CyanFg);
        println!("{}\n", id);

        colorize_print(format!("{} ", method), Colors::CyanFg);
        println!("{}", route);

        Self::log_h2("Headers");
        Self::log_headers(headers);

        if let Some(req_body) = req_body {
            Self::log_h2("Body");
            Self::log_json(req_body, 0);
        }
    }

    fn log_response(id: Uuid, status_code: StatusCode, duration_ms: u128, res_body: Option<Value>) {
        Self::log_h1("Response");

        colorize_print("Request ID: ", Colors::CyanFg);
        println!("{}\n", id);

        colorize_print("Status Code: ", Colors::CyanFg);

        if status_code.is_success() {
            colorize_println(status_code.to_string(), Colors::GreenFg);
        } else if status_code.is_client_error() {
            colorize_println(status_code.to_string(), Colors::YellowFg);
        } else if status_code.is_server_error() {
            colorize_println(status_code.to_string(), Colors::RedFg);
        } else {
            colorize_println(status_code.to_string(), Colors::MagentaFg);
        }

        colorize_print("Duration: ", Colors::CyanFg);
        colorize_println(format!("{} ms", duration_ms), Colors::BlueFg);

        if let Some(res_body) = res_body {
            Self::log_h2("Body");
            Self::log_json(res_body, 0);
        }
    }

    fn extract_after_src(path: Option<&str>) -> String {
        match path {
            Some(path) => {
                let src_prefix = "src/";

                if let Some(start_index) = path.find(src_prefix) {
                    let start_index = start_index + src_prefix.len();
                    path[start_index..].to_string()
                } else {
                    String::new()
                }
            }
            None => String::new(),
        }
    }

    fn log_error(record: &Record) {
        let hashtags = Self::get_hashtags(6);
        let error_header = format!("\n{} Error {}\n", hashtags, hashtags);
        let file_path = Self::extract_after_src(record.file());
        let line_number = record
            .line()
            .map(|line| line.to_string())
            .unwrap_or_default();

        colorize_println(error_header, Colors::RedFg);
        colorize_println(
            format!("File: {}\nLine Number: {}\n", file_path, line_number),
            Colors::RedFg,
        );
        colorize_println(format!("{}", record.args()), Colors::RedFg);
    }

    fn log_debug(record: &Record) {
        let hashtags = Self::get_hashtags(6);
        let debug_header = format!("\n{} Debug {}\n", hashtags, hashtags);
        let file_path = Self::extract_after_src(record.file());
        let line_number = record
            .line()
            .map(|line| line.to_string())
            .unwrap_or_default();

        colorize_println(debug_header, Colors::YellowFg);
        colorize_println(
            format!("File: {}\nLine Number: {}\n", file_path, line_number),
            Colors::YellowFg,
        );
        colorize_println(format!("{}", record.args()), Colors::YellowFg);
    }
}

impl Log for Logger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let blue = "\x1b[34m";
        let purple = "\x1b[35m";
        let clear = "\x1b[0m";
        let file_path = Self::extract_after_src(record.file());
        let line_number = record
            .line()
            .map(|line| line.to_string())
            .unwrap_or_default();

        if file_path.contains("index.crates") {
            return;
        }

        match record.level() {
            Level::Info => println!("\n{}{}{}", blue, record.args(), clear),
            Level::Error => Self::log_error(record),
            Level::Debug => Self::log_debug(record),
            _ => println!(
                "\n{}File: {}{}\n{}Line Number: {}{}\n{}",
                purple,
                file_path,
                clear,
                purple,
                line_number,
                clear,
                record.args(),
            ),
        }
    }

    fn flush(&self) {}
}

#[cfg(test)]
mod tests {
    //! Behavior tests for logging redaction and middleware safety.
    //!
    //! These tests cover:
    //! - Sensitive key/header redaction
    //! - Request payload preservation after middleware inspection
    //! - Response payload preservation after middleware inspection
    //! - Non-JSON compatibility and body-disabled behavior

    use actix_cors::Cors;
    use actix_web::{
        App, HttpResponse,
        http::{Method, StatusCode, header},
        middleware::from_fn,
        post, test, web,
    };
    use serde_json::json;

    use super::{HttpLoggingConfig, Logger};

    #[actix_web::test]
    async fn redact_json_masks_sensitive_keys() {
        let mut value = json!({
            "password": "secret",
            "nested": {
                "auth_code": "123456",
                "name": "Taylor"
            }
        });

        Logger::redact_json_value(&mut value);

        assert_eq!(value["password"], "(hidden)");
        assert_eq!(value["nested"]["auth_code"], "(hidden)");
        assert_eq!(value["nested"]["name"], "Taylor");
    }

    #[actix_web::test]
    async fn sanitize_header_value_masks_sensitive_headers() {
        assert_eq!(
            Logger::sanitize_header_value("Authorization", "Bearer abc"),
            "(hidden)"
        );
        assert_eq!(Logger::sanitize_header_value("X-Request-Id", "123"), "123");
    }

    #[actix_web::test]
    async fn parse_redacted_json_returns_none_for_invalid_json() {
        let value = Logger::parse_redacted_json(b"not-json");
        assert!(value.is_none());
    }

    #[actix_web::test]
    async fn response_body_logging_allows_json_without_content_length() {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let config = HttpLoggingConfig {
            body_enabled: true,
            max_body_bytes: 16_384,
        };

        assert!(Logger::should_attempt_response_body_logging(
            &headers, &config
        ));
    }

    #[post("/echo-json")]
    async fn echo_json(body: web::Bytes) -> HttpResponse {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }

    #[post("/echo-text")]
    async fn echo_text(body: String) -> HttpResponse {
        HttpResponse::Ok().body(body)
    }

    #[post("/always-bad-request")]
    async fn always_bad_request() -> Result<HttpResponse, actix_web::Error> {
        Err(actix_web::error::ErrorBadRequest("invalid request"))
    }

    #[actix_web::test]
    async fn middleware_preserves_request_and_response_json_body() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(HttpLoggingConfig {
                    body_enabled: true,
                    max_body_bytes: 16_384,
                }))
                .wrap(from_fn(Logger::log_request_and_response))
                .service(echo_json),
        )
        .await;

        let payload = json!({
            "password": "super-secret",
            "note": "keep"
        });

        let request = test::TestRequest::post()
            .uri("/echo-json")
            .set_json(&payload)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(body, payload);
    }

    #[actix_web::test]
    async fn middleware_handles_non_json_bodies_without_regression() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(HttpLoggingConfig {
                    body_enabled: true,
                    max_body_bytes: 16_384,
                }))
                .wrap(from_fn(Logger::log_request_and_response))
                .service(echo_text),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/echo-text")
            .set_payload("plain-text-body")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = test::read_body(response).await;
        assert_eq!(body, web::Bytes::from_static(b"plain-text-body"));
    }

    #[actix_web::test]
    async fn middleware_works_when_body_logging_is_disabled() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(HttpLoggingConfig {
                    body_enabled: false,
                    max_body_bytes: 16_384,
                }))
                .wrap(from_fn(Logger::log_request_and_response))
                .service(echo_json),
        )
        .await;

        let payload = json!({ "message": "hello" });
        let request = test::TestRequest::post()
            .uri("/echo-json")
            .set_json(&payload)
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(body, payload);
    }

    #[actix_web::test]
    async fn middleware_preserves_error_status_for_handler_errors() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(HttpLoggingConfig {
                    body_enabled: true,
                    max_body_bytes: 16_384,
                }))
                .wrap(from_fn(Logger::log_request_and_response))
                .service(always_bad_request),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/always-bad-request")
            .to_request();

        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn middleware_works_for_cors_preflight_options_requests() {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods(vec!["GET", "PUT", "POST", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .supports_credentials();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(HttpLoggingConfig {
                    body_enabled: true,
                    max_body_bytes: 16_384,
                }))
                .wrap(cors)
                .wrap(from_fn(Logger::log_request_and_response))
                .service(echo_json),
        )
        .await;

        let request = test::TestRequest::default()
            .method(Method::OPTIONS)
            .uri("/echo-json")
            .insert_header((header::ORIGIN, "http://localhost:3000"))
            .insert_header((header::ACCESS_CONTROL_REQUEST_METHOD, "POST"))
            .to_request();

        let response = test::call_service(&app, request).await;
        assert!(response.status().is_success());
    }
}
