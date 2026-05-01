//! HTTP relay that forwards browser logs into the dev orchestrator stream.
//!
//! This module exposes a small Axum server used by Trunk proxying so web logs
//! appear in the same TUI as API and build output.

use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use goggin_rs_logger::{RelayLogPayload, format_relay_line, split_formatted_lines};
use tokio::{
    net::TcpListener,
    sync::{Mutex, mpsc, oneshot},
    task::JoinHandle,
};

use super::log_store::{LogEntry, Service};

/// Defines the socket address used by the relay server.
pub const WEB_LOG_RELAY_ADDR: &str = "127.0.0.1:9777";
/// Defines the backend URL used by Trunk proxy configuration.
pub const WEB_LOG_RELAY_BACKEND_URL: &str = "http://127.0.0.1:9777";
/// Defines the proxied endpoint path used by browser log posts.
pub const WEB_LOG_RELAY_PROXY_PATH: &str = "/_giglog/web-log";

/// Stores shared state for relay handlers.
#[derive(Clone)]
struct RelayState {
    /// Sends converted browser logs into the orchestrator log stream.
    web_log_tx: mpsc::Sender<LogEntry>,
    /// Controls verbose multi-line formatting behavior.
    verbose: bool,
    /// Serializes multiline emission so grouped lines stay contiguous.
    emit_lock: Arc<Mutex<()>>,
}

/// Owns the relay task and graceful shutdown channel.
pub struct WebLogRelay {
    /// Sends shutdown signal to the relay server task.
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Runs the Axum server until shutdown.
    task: JoinHandle<()>,
}

impl WebLogRelay {
    /// Shuts down the relay task and waits for completion.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
        let _ = self.task.await;
    }
}

/// Starts the web log relay server.
///
/// # Arguments
///
/// * `web_log_tx` — Sender used to forward web log entries.
///
/// # Returns
///
/// A [`WebLogRelay`] handle for graceful shutdown.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the relay socket cannot bind.
pub async fn start(web_log_tx: mpsc::Sender<LogEntry>) -> Result<WebLogRelay> {
    let listener = TcpListener::bind(WEB_LOG_RELAY_ADDR)
        .await
        .with_context(|| format!("Failed to bind web log relay on {WEB_LOG_RELAY_ADDR}"))?;

    let app = build_relay_router(web_log_tx.clone(), read_verbose_from_env());

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        let server = axum::serve(listener, app).with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        });

        if let Err(error) = server.await {
            let _ = web_log_tx
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

/// Builds the relay router using the shared logger receiver.
///
/// # Arguments
///
/// * `web_log_tx` — Sender used to forward web log entries.
/// * `verbose` — Enables expanded multi-line relay formatting.
///
/// # Returns
///
/// A [`Router`] accepting relay log posts.
fn build_relay_router(web_log_tx: mpsc::Sender<LogEntry>, verbose: bool) -> Router {
    let state = RelayState {
        web_log_tx,
        verbose,
        emit_lock: Arc::new(Mutex::new(())),
    };

    Router::new()
        .route("/", post(relay_log))
        .route(WEB_LOG_RELAY_PROXY_PATH, post(relay_log))
        .with_state(state)
}

/// Accepts posted web logs and forwards formatted lines to the orchestrator.
///
/// Mapped to `POST /` and `POST /_giglog/web-log`.
///
/// # Arguments
///
/// * `state` — Shared relay state including output sender and verbosity mode.
/// * `payload` — Posted log payload from the web client.
///
/// # Returns
///
/// A [`StatusCode`] indicating relay acceptance or backpressure failure.
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

/// Reads verbosity configuration from `LOG_VERBOSE`.
///
/// # Returns
///
/// A boolean controlling verbose relay formatting behavior.
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

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, header},
    };
    use serde_json::json;
    use tokio::sync::mpsc;
    use tower::ServiceExt;

    use super::{WEB_LOG_RELAY_PROXY_PATH, build_relay_router};
    use crate::dev::log_store::Service;

    #[tokio::test]
    async fn relay_router_accepts_existing_proxy_path() {
        let (tx, mut rx) = mpsc::channel(8);
        let app = build_relay_router(tx, false);
        let body = serde_json::to_vec(&json!({
            "level": "error",
            "message": "boom",
            "target": "app::module",
            "file": "/tmp/project/src/app/mod.rs",
            "line": 42,
        }))
        .expect("payload should serialize");

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(WEB_LOG_RELAY_PROXY_PATH)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .expect("request should build"),
            )
            .await
            .expect("relay request should route");

        assert_eq!(response.status().as_u16(), 204);

        let entry = rx.recv().await.expect("relay should emit a web log entry");
        assert_eq!(entry.service, Service::Web);
        assert!(entry.line.contains("[ERROR] boom"));
    }

    #[tokio::test]
    async fn relay_router_reports_unavailable_when_log_channel_is_closed() {
        let (tx, rx) = mpsc::channel(8);
        drop(rx);

        let app = build_relay_router(tx, false);
        let body = serde_json::to_vec(&json!({
            "level": "error",
            "message": "boom",
            "target": "app::module",
            "file": "/tmp/project/src/app/mod.rs",
            "line": 42,
        }))
        .expect("payload should serialize");

        let response = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(WEB_LOG_RELAY_PROXY_PATH)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .expect("request should build"),
            )
            .await
            .expect("relay request should route");

        assert_eq!(response.status().as_u16(), 503);
    }
}
