mod app;
use app::App;
mod logging;

use gig_log_common::logging::{debug, info, is_off};

const DEFAULT_WEB_LOG_LEVEL: &str = if cfg!(debug_assertions) {
    "debug"
} else {
    "off"
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

    leptos::mount::mount_to_body(App);

    info!("GigLog web app mounted");
}
