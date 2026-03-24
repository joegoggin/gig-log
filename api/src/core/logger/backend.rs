use std::{
    env,
    sync::atomic::{AtomicBool, Ordering},
};

use colorized::{Colors, colorize_println};
use log::{Level, LevelFilter, Log, Metadata, Record, set_logger, set_max_level};

use super::formatting::{extract_after_src, get_hashtags, log_debug, log_error};

pub struct Logger;

static LOGGER: Logger = Logger;
static LOG_VERBOSE: AtomicBool = AtomicBool::new(true);

impl Logger {
    pub fn setup_logging(log_level: &str, verbose: bool) {
        LOG_VERBOSE.store(verbose, Ordering::Relaxed);

        let _ = set_logger(&LOGGER);
        set_max_level(Self::parse_level_filter(log_level));
    }

    pub fn setup_logging_from_env() {
        dotenvy::dotenv().ok();

        let level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let verbose = env::var("LOG_VERBOSE")
            .ok()
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "true" | "1" | "yes" | "on"
                )
            })
            .unwrap_or(true);

        Self::setup_logging(&level, verbose);
    }

    pub fn is_verbose() -> bool {
        LOG_VERBOSE.load(Ordering::Relaxed)
    }

    pub fn log_success(message: &str) {
        if Self::is_verbose() {
            let hashtags = get_hashtags(6);
            let message = format!("\n{} {} {}\n", hashtags, message, hashtags);
            colorize_println(message, Colors::GreenFg);
            return;
        }

        colorize_println(message, Colors::GreenFg);
    }

    pub fn log_message(message: &str) {
        if Self::is_verbose() {
            let hashtags = get_hashtags(6);
            let message = format!("\n{} {} {}\n", hashtags, message, hashtags);
            colorize_println(message, Colors::BlueFg);
            return;
        }

        colorize_println(message, Colors::BlueFg);
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
}

impl Log for Logger {
    fn enabled(&self, _: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let file_path = extract_after_src(record.file());

        if file_path.contains("index.crates") {
            return;
        }

        if !Logger::is_verbose() {
            match record.level() {
                Level::Error => {
                    colorize_println(format!("[ERROR] {}", record.args()), Colors::RedFg)
                }
                Level::Warn => {
                    colorize_println(format!("[WARN] {}", record.args()), Colors::YellowFg)
                }
                Level::Info => println!("{}", record.args()),
                Level::Debug => {
                    colorize_println(format!("[DEBUG] {}", record.args()), Colors::BlueFg)
                }
                Level::Trace => {
                    colorize_println(format!("[TRACE] {}", record.args()), Colors::MagentaFg)
                }
            }

            return;
        }

        let blue = "\x1b[34m";
        let purple = "\x1b[35m";
        let clear = "\x1b[0m";
        let line_number = record
            .line()
            .map(|line| line.to_string())
            .unwrap_or_default();

        match record.level() {
            Level::Info => println!("\n{}{}{}", blue, record.args(), clear),
            Level::Error => log_error(record),
            Level::Debug => log_debug(record),
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
