use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
};

use super::log_store::{LogEntry, Service};

pub const WEB_LOG_RELAY_ADDR: &str = "127.0.0.1:9777";
pub const WEB_LOG_RELAY_BACKEND_URL: &str = "http://127.0.0.1:9777";
pub const WEB_LOG_RELAY_PROXY_PATH: &str = "/_giglog/web-log";

const ANSI_BLUE: &str = "\x1b[34m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_MAGENTA: &str = "\x1b[35m";
const ANSI_PURPLE: &str = "\x1b[35m";
const ANSI_CLEAR: &str = "\x1b[0m";

const MESSAGE_TARGET: &str = "gig_log::message";
const SUCCESS_TARGET: &str = "gig_log::success";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SemanticLogKind {
    Message,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebLogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl WebLogLevel {
    fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "error" => Self::Error,
            "warn" | "warning" => Self::Warn,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            "info" => Self::Info,
            _ => Self::Info,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RelayLogPayload {
    level: String,
    message: String,
    target: Option<String>,
    file: Option<String>,
    line: Option<u32>,
}

#[derive(Clone)]
struct RelayState {
    web_log_tx: mpsc::Sender<LogEntry>,
    verbose: bool,
    emit_lock: Arc<Mutex<()>>,
}

pub struct WebLogRelay {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

impl WebLogRelay {
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let _ = self.task.await;
    }
}

pub async fn start(web_log_tx: mpsc::Sender<LogEntry>) -> Result<WebLogRelay> {
    let listener = TcpListener::bind(WEB_LOG_RELAY_ADDR)
        .await
        .with_context(|| format!("Failed to bind web log relay on {WEB_LOG_RELAY_ADDR}"))?;

    let state = RelayState {
        web_log_tx,
        verbose: read_verbose_from_env(),
        emit_lock: Arc::new(Mutex::new(())),
    };

    let app = Router::new()
        .route("/", post(relay_log))
        .route(WEB_LOG_RELAY_PROXY_PATH, post(relay_log))
        .with_state(state.clone());

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        if let Err(error) = server.await {
            let _ = state
                .web_log_tx
                .send(LogEntry {
                    service: Service::System,
                    line: format!("Web log relay stopped unexpectedly: {error}"),
                })
                .await;
        }
    });

    Ok(WebLogRelay {
        shutdown_tx: Some(shutdown_tx),
        task,
    })
}

async fn relay_log(
    State(state): State<RelayState>,
    Json(payload): Json<RelayLogPayload>,
) -> StatusCode {
    let _emit_guard = state.emit_lock.lock().await;
    let formatted = format_relay_line(&payload, state.verbose);

    for line in split_formatted_lines(&formatted) {
        if state
            .web_log_tx
            .send(LogEntry {
                service: Service::Web,
                line,
            })
            .await
            .is_err()
        {
            return StatusCode::SERVICE_UNAVAILABLE;
        }
    }

    StatusCode::NO_CONTENT
}

fn read_verbose_from_env() -> bool {
    std::env::var("LOG_VERBOSE")
        .ok()
        .map(|value| match value.trim().to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => true,
        })
        .unwrap_or(true)
}

fn format_relay_line(payload: &RelayLogPayload, verbose: bool) -> String {
    let level = WebLogLevel::parse(&payload.level);
    let message = payload.message.trim_end().to_string();
    let semantic_kind = parse_semantic_log_kind(payload.target.as_deref());

    if let Some(semantic_kind) = semantic_kind {
        return format_semantic_line(semantic_kind, &message, verbose);
    }

    let file = payload
        .file
        .as_deref()
        .map(extract_after_src)
        .filter(|value| !value.is_empty())
        .or_else(|| payload.target.clone())
        .unwrap_or_default();
    let line_number = payload
        .line
        .map(|line| line.to_string())
        .unwrap_or_default();

    if !verbose {
        return format_non_verbose(level, &message);
    }

    match level {
        WebLogLevel::Info => format!("\n{ANSI_BLUE}{message}{ANSI_CLEAR}"),
        WebLogLevel::Error => format_error_like("Error", ANSI_RED, &file, &line_number, &message),
        WebLogLevel::Debug => {
            format_error_like("Debug", ANSI_YELLOW, &file, &line_number, &message)
        }
        WebLogLevel::Warn | WebLogLevel::Trace => format!(
            "\n{ANSI_PURPLE}File: {file}{ANSI_CLEAR}\n{ANSI_PURPLE}Line Number: {line_number}{ANSI_CLEAR}\n{message}"
        ),
    }
}

fn parse_semantic_log_kind(target: Option<&str>) -> Option<SemanticLogKind> {
    match target {
        Some(MESSAGE_TARGET) => Some(SemanticLogKind::Message),
        Some(SUCCESS_TARGET) => Some(SemanticLogKind::Success),
        _ => None,
    }
}

fn format_semantic_line(kind: SemanticLogKind, message: &str, verbose: bool) -> String {
    let (color, hashtags) = match kind {
        SemanticLogKind::Message => (ANSI_BLUE, "######"),
        SemanticLogKind::Success => (ANSI_GREEN, "######"),
    };

    if verbose {
        return format!("\n{color}{hashtags} {message} {hashtags}\n{ANSI_CLEAR}");
    }

    format!("{color}{message}{ANSI_CLEAR}")
}

fn format_non_verbose(level: WebLogLevel, message: &str) -> String {
    match level {
        WebLogLevel::Error => format!("{ANSI_RED}[ERROR] {message}{ANSI_CLEAR}"),
        WebLogLevel::Warn => format!("{ANSI_YELLOW}[WARN] {message}{ANSI_CLEAR}"),
        WebLogLevel::Info => message.to_string(),
        WebLogLevel::Debug => format!("{ANSI_BLUE}[DEBUG] {message}{ANSI_CLEAR}"),
        WebLogLevel::Trace => format!("{ANSI_MAGENTA}[TRACE] {message}{ANSI_CLEAR}"),
    }
}

fn format_error_like(
    title: &str,
    color: &str,
    file: &str,
    line_number: &str,
    message: &str,
) -> String {
    let hashtags = "######";
    format!(
        "\n{color}{hashtags} {title} {hashtags}{ANSI_CLEAR}\n{color}File: {file}{ANSI_CLEAR}\n{color}Line Number: {line_number}{ANSI_CLEAR}\n\n{color}{message}{ANSI_CLEAR}"
    )
}

fn split_formatted_lines(formatted: &str) -> Vec<String> {
    formatted.split('\n').map(str::to_string).collect()
}

fn extract_after_src(path: &str) -> String {
    let src_prefix = "src/";
    if let Some(start_index) = path.find(src_prefix) {
        return path[start_index + src_prefix.len()..].to_string();
    }

    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        RelayLogPayload, WebLogLevel, extract_after_src, format_non_verbose, format_relay_line,
        split_formatted_lines,
    };

    #[test]
    fn parse_level_defaults_to_info() {
        assert_eq!(WebLogLevel::parse("wat"), WebLogLevel::Info);
        assert_eq!(WebLogLevel::parse("warn"), WebLogLevel::Warn);
    }

    #[test]
    fn extract_after_src_returns_relative_suffix() {
        assert_eq!(
            extract_after_src("/tmp/project/src/main.rs"),
            "main.rs".to_string()
        );
    }

    #[test]
    fn non_verbose_error_uses_api_style_prefix() {
        let formatted = format_non_verbose(WebLogLevel::Error, "boom");
        assert!(formatted.contains("[ERROR] boom"));
        assert!(formatted.contains("\u{1b}[31m"));
    }

    #[test]
    fn verbose_debug_uses_banner_format() {
        let payload = RelayLogPayload {
            level: "debug".to_string(),
            message: "test log".to_string(),
            target: Some("app::module".to_string()),
            file: Some("/tmp/project/src/app/mod.rs".to_string()),
            line: Some(42),
        };

        let formatted = format_relay_line(&payload, true);
        assert!(formatted.contains("###### Debug ######"));
        assert!(formatted.contains("File: app/mod.rs"));
        assert!(formatted.contains("Line Number: 42"));
        assert!(formatted.contains("test log"));
    }

    #[test]
    fn split_formatted_lines_preserves_blank_lines() {
        let formatted = "\n\x1b[31m###### Error ######\x1b[0m\n\n\x1b[31mboom\x1b[0m\n";
        let lines = split_formatted_lines(formatted);

        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], "");
        assert!(lines[1].contains("###### Error ######"));
        assert_eq!(lines[2], "");
        assert!(lines[3].contains("boom"));
        assert_eq!(lines[4], "");
    }

    #[test]
    fn semantic_success_uses_api_style_formatting() {
        let payload = RelayLogPayload {
            level: "info".to_string(),
            message: "Database connection established".to_string(),
            target: Some("gig_log::success".to_string()),
            file: Some("/tmp/project/src/app/mod.rs".to_string()),
            line: Some(42),
        };

        let verbose = format_relay_line(&payload, true);
        assert!(verbose.contains("###### Database connection established ######"));
        assert!(verbose.contains("\u{1b}[32m"));

        let non_verbose = format_relay_line(&payload, false);
        assert!(non_verbose.contains("Database connection established"));
        assert!(non_verbose.contains("\u{1b}[32m"));
    }
}
