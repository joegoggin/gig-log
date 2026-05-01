//! Browser entry point for the GigLog frontend.
//!
//! This binary initializes browser logging and mounts the root Leptos
//! application component into the document body.
//!
//! # Modules
//!
//! - [`app`] — Root router component and route registrations.
//! - [`logging`] — Browser log relay initialization utilities.

/// Defines the root application component module.
mod app;
use app::App;

use goggin_rs_logger::{
    WebLoggerConfig, init_web_logging_with_config, is_off, log_message, log_success,
};

const DEFAULT_WEB_LOG_LEVEL: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "off"
};
const WEB_LOG_RELAY_ENDPOINT: &str = "/_giglog/web-log";

/// Initializes browser logging with the default level fallback.
///
/// Calls [`logging::init_web_logging`] and emits startup logs when logging is
/// enabled.
fn init_web_logging() {
    let logger_config =
        WebLoggerConfig::new(DEFAULT_WEB_LOG_LEVEL).with_endpoint(WEB_LOG_RELAY_ENDPOINT);
    let logger_config = match init_web_logging_with_config(logger_config) {
        Ok(config) => config,
        Err(_) => return,
    };

    if is_off(logger_config.level_filter) {
        return;
    }

    log_message(&format!(
        "Web logger initialized with WEB_LOG_LEVEL='{}' ({:?})",
        logger_config.configured_level, logger_config.level_filter
    ));
    log_message("Mounting GigLog web app");
}

/// Starts the frontend application runtime.
///
/// Initializes logging and mounts [`App`] into the page body.
fn main() {
    init_web_logging();

    leptos::mount::mount_to_body(App);

    log_success("GigLog web app mounted");
}
