use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use gig_log_common::logging::{is_off, parse_level_filter};
use gloo_net::http::Request;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;

const WEB_LOG_RELAY_ENDPOINT: &str = "/_giglog/web-log";

#[derive(Debug, Clone, Copy)]
pub struct WebLoggerConfig {
    pub configured_level: &'static str,
    pub level_filter: LevelFilter,
}

struct WebRelayLogger;

static WEB_RELAY_LOGGER: WebRelayLogger = WebRelayLogger;

thread_local! {
    static RELAY_QUEUE: RefCell<VecDeque<RelayLogPayload>> = RefCell::new(VecDeque::new());
    static RELAY_SENDING: Cell<bool> = const { Cell::new(false) };
}

#[derive(Debug, Serialize)]
struct RelayLogPayload {
    level: String,
    message: String,
    target: Option<String>,
    file: Option<String>,
    line: Option<u32>,
}

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
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

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

    fn flush(&self) {}
}

fn enqueue_payload(payload: RelayLogPayload) {
    RELAY_QUEUE.with(|queue| {
        queue.borrow_mut().push_back(payload);
    });

    start_sender_if_needed();
}

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

async fn send_payload(payload: RelayLogPayload) {
    let request = match Request::post(WEB_LOG_RELAY_ENDPOINT).json(&payload) {
        Ok(request) => request,
        Err(_) => return,
    };

    let _ = request.send().await;
}
