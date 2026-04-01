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
/// Defines browser logging initialization helpers.
mod logging;

use gig_log_common::logging::{is_off, log_message, log_success};

const DEFAULT_WEB_LOG_LEVEL: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "off"
};

/// Initializes browser logging with the default level fallback.
///
/// Calls [`logging::init_web_logging`] and emits startup logs when logging is
/// enabled.
fn init_web_logging() {
    let logger_config = match logging::init_web_logging(DEFAULT_WEB_LOG_LEVEL) {
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
