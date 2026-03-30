//! Shared logging utilities and re-exports.
//!
//! This module re-exports the [`log`] facade macros for convenience and provides
//! helper functions for parsing and converting log level configuration values.

use log::{Level, LevelFilter};

/// Convenience re-exports of the [`log`] facade macros.
pub use log::{debug, error, info, trace, warn};

/// Log target for semantic informational messages.
const MESSAGE_TARGET: &str = "gig_log::message";
/// Log target for semantic success messages.
const SUCCESS_TARGET: &str = "gig_log::success";

/// Parses a string into a [`LevelFilter`].
///
/// Matching is case-insensitive and trims surrounding whitespace.
/// Unrecognized values default to [`LevelFilter::Info`].
///
/// # Arguments
///
/// * `log_level` — The log level string (e.g. `"debug"`, `"info"`).
///
/// # Returns
///
/// The corresponding [`LevelFilter`], or [`LevelFilter::Info`] for
/// unrecognized values.
pub fn parse_level_filter(log_level: &str) -> LevelFilter {
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

/// Converts a [`LevelFilter`] to a [`Level`].
///
/// Maps [`LevelFilter::Off`] to [`Level::Error`] so that a logger can still be
/// initialized even when logging is disabled.
///
/// # Arguments
///
/// * `level_filter` — The filter to convert.
///
/// # Returns
///
/// The corresponding [`Level`], with [`LevelFilter::Off`] mapped to
/// [`Level::Error`].
pub fn level_for_logger(level_filter: LevelFilter) -> Level {
    match level_filter {
        LevelFilter::Off | LevelFilter::Error => Level::Error,
        LevelFilter::Warn => Level::Warn,
        LevelFilter::Info => Level::Info,
        LevelFilter::Debug => Level::Debug,
        LevelFilter::Trace => Level::Trace,
    }
}

/// Returns `true` if the given filter is [`LevelFilter::Off`].
///
/// # Arguments
///
/// * `level_filter` — The filter to check.
///
/// # Returns
///
/// `true` if the filter is [`LevelFilter::Off`].
pub fn is_off(level_filter: LevelFilter) -> bool {
    matches!(level_filter, LevelFilter::Off)
}

pub fn log_message(message: &str) {
    log::info!(target: MESSAGE_TARGET, "{}", message);
}

pub fn log_success(message: &str) {
    log::info!(target: SUCCESS_TARGET, "{}", message);
}

#[cfg(test)]
mod tests {
    use log::{Level, LevelFilter};

    use super::{is_off, level_for_logger, parse_level_filter};

    #[test]
    fn parse_level_filter_handles_expected_values() {
        assert_eq!(parse_level_filter("off"), LevelFilter::Off);
        assert_eq!(parse_level_filter("error"), LevelFilter::Error);
        assert_eq!(parse_level_filter("warn"), LevelFilter::Warn);
        assert_eq!(parse_level_filter("info"), LevelFilter::Info);
        assert_eq!(parse_level_filter("debug"), LevelFilter::Debug);
        assert_eq!(parse_level_filter("trace"), LevelFilter::Trace);
        assert_eq!(parse_level_filter("invalid"), LevelFilter::Info);
    }

    #[test]
    fn parse_level_filter_is_case_and_whitespace_insensitive() {
        assert_eq!(parse_level_filter(" DEBUG "), LevelFilter::Debug);
    }

    #[test]
    fn level_for_logger_maps_off_to_error_for_init() {
        assert_eq!(level_for_logger(LevelFilter::Off), Level::Error);
        assert_eq!(level_for_logger(LevelFilter::Info), Level::Info);
    }

    #[test]
    fn is_off_detects_only_off() {
        assert!(is_off(LevelFilter::Off));
        assert!(!is_off(LevelFilter::Error));
    }
}
