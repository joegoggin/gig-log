//! Browser logging utilities for the GigLog frontend.
//!
//! This module configures a relay logger that captures frontend log records and
//! forwards them to the backend log relay endpoint.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use gig_log_common::logging::{is_off, parse_level_filter};
use gloo_net::http::Request;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;

const WEB_LOG_RELAY_ENDPOINT: &str = "/_giglog/web-log";
const MAX_RELAY_QUEUE_LEN: usize = 1024;

/// Stores resolved browser logger configuration values.
#[derive(Debug, Clone, Copy)]
pub struct WebLoggerConfig {
    /// Stores the configured level string used for parsing.
    pub configured_level: &'static str,
    /// Stores the parsed [`LevelFilter`] used by the logger.
    pub level_filter: LevelFilter,
}

/// Implements a [`Log`] sink that relays frontend records over HTTP.
struct WebRelayLogger;

static WEB_RELAY_LOGGER: WebRelayLogger = WebRelayLogger;

thread_local! {
    static RELAY_QUEUE: RefCell<VecDeque<RelayLogPayload>> = RefCell::new(VecDeque::new());
    static RELAY_SENDING: Cell<bool> = const { Cell::new(false) };
}

/// Defines a serialized log payload sent to the relay endpoint.
#[derive(Debug, Serialize)]
struct RelayLogPayload {
    /// Stores the lowercased log level name.
    level: String,
    /// Stores the formatted log message.
    message: String,
    /// Stores the log target when provided.
    target: Option<String>,
    /// Stores the source file path when available.
    file: Option<String>,
    /// Stores the source line number when available.
    line: Option<u32>,
}

/// Initializes browser logging for the frontend runtime.
///
/// Resolves the log level from `WEB_LOG_LEVEL`, updates the max log level, and
/// installs the relay logger when the level is not `off`.
///
/// # Arguments
///
/// * `default_level` — Fallback log level when `WEB_LOG_LEVEL` is unset.
///
/// # Returns
///
/// A [`Result`] containing the resolved [`WebLoggerConfig`] on success.
///
/// # Errors
///
/// Returns a [`SetLoggerError`] if the global logger has already been set.
pub fn init_web_logging(default_level: &'static str) -> Result<WebLoggerConfig, SetLoggerError> {
    let configured_level = option_env!("WEB_LOG_LEVEL").unwrap_or(default_level);
    let level_filter = parse_level_filter(configured_level);

    log::set_max_level(level_filter);

    if is_off(level_filter) {
        return Ok(WebLoggerConfig {
            configured_level,
            level_filter,
        });
    }

    log::set_logger(&WEB_RELAY_LOGGER)?;

    Ok(WebLoggerConfig {
        configured_level,
        level_filter,
    })
}

impl Log for WebRelayLogger {
    /// Returns whether the logger should accept the record metadata.
    ///
    /// # Arguments
    ///
    /// * `_metadata` — Metadata associated with the candidate log record.
    ///
    /// # Returns
    ///
    /// A [`bool`] indicating whether the record is enabled.
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    /// Relays a log record by enqueuing a serialized payload.
    ///
    /// # Arguments
    ///
    /// * `record` — Log record emitted through the [`log`] facade.
    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let payload = RelayLogPayload {
            level: record.level().to_string().to_ascii_lowercase(),
            message: record.args().to_string(),
            target: Some(record.target().to_string()),
            file: record.file().map(str::to_string),
            line: record.line(),
        };

        enqueue_payload(payload);
    }

    /// Flushes buffered logs for the relay logger.
    fn flush(&self) {}
}

/// Enqueues a relay payload and starts the sender task when idle.
///
/// # Arguments
///
/// * `payload` — Serialized payload to queue for transmission.
fn enqueue_payload(payload: RelayLogPayload) {
    RELAY_QUEUE.with(|queue| {
        let mut queue = queue.borrow_mut();
        if queue.len() >= MAX_RELAY_QUEUE_LEN {
            let _ = queue.pop_front();
        }
        queue.push_back(payload);
    });

    start_sender_if_needed();
}

/// Starts the async relay sender loop when it is not already running.
fn start_sender_if_needed() {
    let should_start = RELAY_SENDING.with(|sending| {
        if sending.get() {
            false
        } else {
            sending.set(true);
            true
        }
    });

    if !should_start {
        return;
    }

    spawn_local(async {
        loop {
            let next = RELAY_QUEUE.with(|queue| queue.borrow_mut().pop_front());
            match next {
                Some(payload) => send_payload(payload).await,
                None => {
                    RELAY_SENDING.with(|sending| sending.set(false));
                    break;
                }
            }
        }
    });
}

/// Sends a single relay payload to the backend endpoint.
///
/// # Arguments
///
/// * `payload` — Serialized log payload to transmit.
async fn send_payload(payload: RelayLogPayload) {
    let request = match Request::post(WEB_LOG_RELAY_ENDPOINT).json(&payload) {
        Ok(request) => request,
        Err(_) => return,
    };

    let _ = request.send().await;
}
