mod logging;

use gig_log_common::logging::{debug, info, is_off};
use leptos::prelude::*;
use log::error;

const DEFAULT_WEB_LOG_LEVEL: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "info"
};

fn init_web_logging() {
    let logger_config = match logging::init_web_logging(DEFAULT_WEB_LOG_LEVEL) {
        Ok(config) => config,
        Err(_) => return,
    };

    if is_off(logger_config.level_filter) {
        return;
    }

    info!(
        "Web logger initialized with WEB_LOG_LEVEL='{}' ({:?})",
        logger_config.configured_level, logger_config.level_filter
    );
    debug!("Mounting GigLog web app");
}

fn main() {
    init_web_logging();

    info!("This is an info message");
    debug!("This is a debug message");
    error!("this is an error message");

    leptos::mount::mount_to_body(|| view! { <h1>"Hello World!"</h1> });

    info!("GigLog web app mounted");
}
