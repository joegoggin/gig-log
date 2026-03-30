//! Process lifecycle and command execution helpers for the dev orchestrator.
//!
//! This module manages long-running services, short-lived build jobs, and log
//! stream normalization so terminal rendering preserves ANSI styling.

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use super::{
    log_store::{LogEntry, Service},
    web_log_relay::{WEB_LOG_RELAY_BACKEND_URL, WEB_LOG_RELAY_PROXY_PATH},
};

/// Tracks a long-running orchestrated service process.
pub struct ServiceProcess {
    /// Identifies which service this child process represents.
    pub service: Service,
    /// Owns the spawned child process handle.
    pub child: Child,
    /// Stores the process group id used for group signaling.
    process_group: u32,
}

impl ServiceProcess {
    /// Spawns a long-running service process and streams logs.
    ///
    /// # Arguments
    ///
    /// * `service` — Service variant to spawn.
    ///
    /// # Returns
    ///
    /// A [`ServiceProcess`] and receiver for emitted [`LogEntry`] lines.
    ///
    /// # Errors
    ///
    /// Returns an [`anyhow::Error`] if command resolution or process spawn fails.
    pub fn spawn(service: Service) -> Result<(Self, mpsc::Receiver<LogEntry>)> {
        let (cmd, args, working_dir) = service_command(service)?;
        let (tx, rx) = mpsc::channel(256);

        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        if matches!(service, Service::Web) {
            command.env("SASS_PATH", "styles");
        }

        command.env("CARGO_TERM_COLOR", "always");
        command.env("CLICOLOR_FORCE", "1");
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.kill_on_drop(true);

        unsafe {
            command.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let mut child = command
            .spawn()
            .with_context(|| format!("Failed to spawn {} service", service.label()))?;

        let pid = child.id().unwrap_or(0);

        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            tokio::spawn(read_lines(stdout, service, tx));
        }

        if let Some(stderr) = child.stderr.take() {
            let tx = tx.clone();
            tokio::spawn(read_lines(stderr, service, tx));
        }

        Ok((
            Self {
                service,
                child,
                process_group: pid,
            },
            rx,
        ))
    }

    /// Shuts down the process group with a graceful timeout.
    pub async fn shutdown(&mut self) {
        self.shutdown_with_timeout(Duration::from_secs(3)).await;
    }

    /// Shuts down the process group with an aggressive timeout.
    pub async fn shutdown_fast(&mut self) {
        self.shutdown_with_timeout(Duration::from_millis(200)).await;
    }

    /// Signals and waits for process termination with fallback kill.
    ///
    /// # Arguments
    ///
    /// * `timeout` — Maximum wait duration after `SIGTERM` before `SIGKILL`.
    async fn shutdown_with_timeout(&mut self, timeout: Duration) {
        self.signal_process_group(libc::SIGTERM);

        let wait_result = tokio::time::timeout(timeout, self.child.wait()).await;

        if wait_result.is_err() {
            self.signal_process_group(libc::SIGKILL);
            let _ = self.child.kill().await;
        }
    }

    /// Sends a Unix signal to the entire process group.
    ///
    /// # Arguments
    ///
    /// * `signal` — POSIX signal number to send.
    fn signal_process_group(&self, signal: i32) {
        if self.process_group > 0 {
            unsafe {
                libc::kill(-(self.process_group as i32), signal);
            }
        }
    }
}

impl Drop for ServiceProcess {
    fn drop(&mut self) {
        let is_running = self.child.try_wait().ok().flatten().is_none();
        if is_running {
            self.signal_process_group(libc::SIGTERM);
            self.signal_process_group(libc::SIGKILL);
        }
    }
}

/// Runs a short-lived command and forwards stdout/stderr as log lines.
///
/// # Arguments
///
/// * `service` — Service channel used for emitted log entries.
/// * `cmd` — Executable name to run.
/// * `args` — Command-line arguments passed to `cmd`.
/// * `working_dir` — Optional working directory override.
/// * `envs` — Optional environment variables to inject.
/// * `tx` — Log sender receiving normalized output lines.
///
/// # Returns
///
/// A boolean indicating whether the command exited successfully.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the command cannot start or wait fails.
pub async fn run_job(
    service: Service,
    cmd: &str,
    args: &[&str],
    working_dir: Option<&str>,
    envs: Option<&[(&str, &str)]>,
    tx: &mpsc::Sender<LogEntry>,
) -> Result<bool> {
    let mut command = Command::new(cmd);
    command.args(args);

    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }

    if let Some(envs) = envs {
        for (key, value) in envs {
            command.env(key, value);
        }
    }

    command.env("CARGO_TERM_COLOR", "always");
    command.env("CLICOLOR_FORCE", "1");
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to start {} command", service.label()))?;

    let mut tasks = Vec::new();

    if let Some(stdout) = child.stdout.take() {
        let tx = tx.clone();
        tasks.push(tokio::spawn(read_lines(stdout, service, tx)));
    }

    if let Some(stderr) = child.stderr.take() {
        let tx = tx.clone();
        tasks.push(tokio::spawn(read_lines(stderr, service, tx)));
    }

    let status = child.wait().await?;

    for task in tasks {
        let _ = task.await;
    }

    Ok(status.success())
}

/// Resolves command metadata for a long-running service.
///
/// # Arguments
///
/// * `service` — Service to resolve into command configuration.
///
/// # Returns
///
/// A tuple of executable, argument list, and optional working directory.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the service is not long-running.
fn service_command(
    service: Service,
) -> Result<(&'static str, Vec<&'static str>, Option<&'static str>)> {
    match service {
        Service::Api => Ok(("cargo", vec!["run", "-p", "gig-log-api"], None)),
        Service::Web => Ok((
            "trunk",
            vec![
                "serve",
                "--proxy-rewrite",
                WEB_LOG_RELAY_PROXY_PATH,
                "--proxy-backend",
                WEB_LOG_RELAY_BACKEND_URL,
            ],
            Some("web/"),
        )),
        _ => anyhow::bail!("{} is not a long-running service", service.label()),
    }
}

/// Reads newline-delimited output from a process stream and forwards logs.
///
/// # Arguments
///
/// * `reader` — Async stream for stdout or stderr.
/// * `service` — Service channel associated with the stream.
/// * `tx` — Sender receiving normalized log entries.
async fn read_lines<R: tokio::io::AsyncRead + Unpin>(
    reader: R,
    service: Service,
    tx: mpsc::Sender<LogEntry>,
) {
    let mut lines = BufReader::new(reader).lines();
    let mut active_ansi_prefix = String::new();

    while let Ok(Some(line)) = lines.next_line().await {
        let Some(line) = normalize_ansi_for_tui(line, &mut active_ansi_prefix) else {
            continue;
        };

        if tx.send(LogEntry { service, line }).await.is_err() {
            break;
        }
    }
}

/// Normalizes ANSI control fragments for line-based TUI rendering.
///
/// # Arguments
///
/// * `line` — Raw line received from process output.
/// * `active_ansi_prefix` — Mutable active SGR prefix carried across lines.
///
/// # Returns
///
/// An optional normalized line suitable for TUI display.
fn normalize_ansi_for_tui(line: String, active_ansi_prefix: &mut String) -> Option<String> {
    if is_ansi_control_only_line(&line) {
        update_active_ansi_prefix(&line, active_ansi_prefix);

        return Some(String::new());
    }

    if line.is_empty() {
        return Some(line);
    }

    let should_prepend_active_prefix =
        !active_ansi_prefix.is_empty() && !starts_with_ansi_sequence(&line);
    let prefix_for_line = if should_prepend_active_prefix {
        Some(active_ansi_prefix.clone())
    } else {
        None
    };

    update_active_ansi_prefix(&line, active_ansi_prefix);

    let Some(prefix_for_line) = prefix_for_line else {
        return Some(line);
    };

    let mut normalized = String::with_capacity(prefix_for_line.len() + line.len());
    normalized.push_str(&prefix_for_line);
    normalized.push_str(&line);

    Some(normalized)
}

/// Detects whether a line starts with an ANSI escape sequence.
///
/// # Arguments
///
/// * `line` — Candidate log line.
///
/// # Returns
///
/// A boolean indicating whether the line starts with `\x1b[`.
fn starts_with_ansi_sequence(line: &str) -> bool {
    line.as_bytes().starts_with(b"\x1b[")
}

/// Updates active ANSI state after processing a rendered line.
///
/// # Arguments
///
/// * `line` — Normalized or raw line containing ANSI sequences.
/// * `active_ansi_prefix` — Active SGR prefix to update in place.
fn update_active_ansi_prefix(line: &str, active_ansi_prefix: &mut String) {
    if starts_with_ansi_sequence(line) {
        active_ansi_prefix.clear();
    }

    let bytes = line.as_bytes();
    let mut index = 0;

    while index + 1 < bytes.len() {
        if bytes[index] != b'\x1b' || bytes[index + 1] != b'[' {
            index += 1;
            continue;
        }

        let sequence_start = index;
        index += 2;

        while index < bytes.len() {
            let byte = bytes[index];
            index += 1;

            if (b'@'..=b'~').contains(&byte) {
                let sequence = &line[sequence_start..index];

                if ansi_sequence_is_reset(sequence) {
                    active_ansi_prefix.clear();
                } else if ansi_sequence_is_sgr(sequence) {
                    active_ansi_prefix.push_str(sequence);
                }

                break;
            }
        }
    }
}

/// Checks whether an ANSI sequence clears terminal styling.
///
/// # Arguments
///
/// * `sequence` — Escape sequence candidate.
///
/// # Returns
///
/// A boolean indicating whether the sequence is an SGR reset.
fn ansi_sequence_is_reset(sequence: &str) -> bool {
    sequence == "\u{1b}[0m" || sequence == "\u{1b}[m"
}

/// Checks whether an ANSI sequence is an SGR style command.
///
/// # Arguments
///
/// * `sequence` — Escape sequence candidate.
///
/// # Returns
///
/// A boolean indicating whether the sequence ends in `m`.
fn ansi_sequence_is_sgr(sequence: &str) -> bool {
    sequence.ends_with('m')
}

/// Detects lines that contain only ANSI control data and no visible text.
///
/// # Arguments
///
/// * `line` — Candidate line to inspect.
///
/// # Returns
///
/// A boolean indicating whether rendered content is empty after stripping ANSI.
fn is_ansi_control_only_line(line: &str) -> bool {
    line.contains('\u{1b}') && strip_ansi_sequences(line).trim().is_empty()
}

/// Removes ANSI escape sequences from a line.
///
/// # Arguments
///
/// * `line` — Input string that may contain ANSI escapes.
///
/// # Returns
///
/// A string with all ANSI escapes removed.
fn strip_ansi_sequences(line: &str) -> String {
    let mut output = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            let _ = chars.next();

            for seq_char in chars.by_ref() {
                if ('@'..='~').contains(&seq_char) {
                    break;
                }
            }

            continue;
        }

        output.push(ch);
    }

    output
}

/// Verifies required external binaries are available in `PATH`.
///
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if `trunk` or `miniserve` is missing.
pub fn check_requirements() -> Result<()> {
    for binary in ["trunk", "miniserve"] {
        if std::process::Command::new("which")
            .arg(binary)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| !s.success())
            .unwrap_or(true)
        {
            anyhow::bail!("{binary} is not installed.");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_ansi_for_tui, service_command};
    use crate::dev::log_store::Service;
    use crate::dev::web_log_relay::{WEB_LOG_RELAY_BACKEND_URL, WEB_LOG_RELAY_PROXY_PATH};

    #[test]
    fn ansi_control_prefix_is_applied_until_reset() {
        let mut active_ansi_prefix = String::new();

        let first = normalize_ansi_for_tui("\u{1b}[34m".to_string(), &mut active_ansi_prefix);
        assert_eq!(first, Some(String::new()));
        assert_eq!(active_ansi_prefix, "\u{1b}[34m");

        let second = normalize_ansi_for_tui("hello".to_string(), &mut active_ansi_prefix);
        assert_eq!(second, Some("\u{1b}[34mhello".to_string()));
        assert_eq!(active_ansi_prefix, "\u{1b}[34m");

        let third = normalize_ansi_for_tui("world".to_string(), &mut active_ansi_prefix);
        assert_eq!(third, Some("\u{1b}[34mworld".to_string()));
        assert_eq!(active_ansi_prefix, "\u{1b}[34m");

        let reset = normalize_ansi_for_tui("\u{1b}[0m".to_string(), &mut active_ansi_prefix);
        assert_eq!(reset, Some(String::new()));
        assert!(active_ansi_prefix.is_empty());

        let after_reset =
            normalize_ansi_for_tui("after reset".to_string(), &mut active_ansi_prefix);
        assert_eq!(after_reset, Some("after reset".to_string()));
    }

    #[test]
    fn ansi_reset_line_clears_pending_prefix() {
        let mut active_ansi_prefix = "\u{1b}[34m".to_string();

        let normalized = normalize_ansi_for_tui("\u{1b}[0m".to_string(), &mut active_ansi_prefix);
        assert_eq!(normalized, Some(String::new()));
        assert!(active_ansi_prefix.is_empty());
    }

    #[test]
    fn inline_reset_clears_active_prefix_for_following_lines() {
        let mut active_ansi_prefix = "\u{1b}[31m".to_string();

        let first = normalize_ansi_for_tui(
            "line one\u{1b}[0m plain".to_string(),
            &mut active_ansi_prefix,
        );
        assert_eq!(first, Some("\u{1b}[31mline one\u{1b}[0m plain".to_string()));
        assert!(active_ansi_prefix.is_empty());

        let second = normalize_ansi_for_tui("line two".to_string(), &mut active_ansi_prefix);
        assert_eq!(second, Some("line two".to_string()));
    }

    #[test]
    fn plain_empty_line_is_preserved() {
        let mut active_ansi_prefix = String::new();
        let normalized = normalize_ansi_for_tui(String::new(), &mut active_ansi_prefix);
        assert_eq!(normalized, Some(String::new()));
    }

    #[test]
    fn web_service_uses_relay_proxy() {
        let (cmd, args, working_dir) = service_command(Service::Web).expect("web command");

        assert_eq!(cmd, "trunk");
        assert_eq!(working_dir, Some("web/"));
        assert!(
            args.windows(2)
                .any(|window| { window == ["--proxy-rewrite", WEB_LOG_RELAY_PROXY_PATH] })
        );
        assert!(
            args.windows(2)
                .any(|window| { window == ["--proxy-backend", WEB_LOG_RELAY_BACKEND_URL] })
        );
    }
}
