//! Development orchestrator for GigLog local workflows.
//!
//! This module coordinates service processes, file watching, docs generation,
//! web-log relay intake, and terminal UI rendering for the `dev` command.
//!
//! # Modules
//!
//! - [`log_store`] — Service labels and in-memory log storage.
//! - [`process`] — Process spawn/shutdown and command execution helpers.
//! - [`tui`] — Terminal event loop and keyboard handling.
//! - [`ui`] — Ratatui rendering primitives for orchestrator views.
//! - [`watcher`] — Filesystem watch classification and debounce batching.
//! - [`web_log_relay`] — HTTP relay for browser logs from the web app.

mod log_store;
mod process;
mod tui;
mod ui;
mod watcher;
mod web_log_relay;

use std::time::Duration;

use anyhow::Result;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::{mpsc, watch};

use log_store::{LogEntry, Service};
use process::{ServiceProcess, check_requirements, run_job};
use tui::TuiEvent;
use watcher::IntentBatch;

/// Defines top-level workspace directories watched for change events.
const WATCH_PATHS: [&str; 4] = ["web", "api", "common", "dev-tools"];
/// Defines debounce duration used to merge nearby watch events.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(400);
/// Defines maximum wait time for a trunk build completion signal.
const WEB_BUILD_TIMEOUT: Duration = Duration::from_secs(60);
/// Defines maximum wait time for the API socket to become reachable.
const API_READY_TIMEOUT: Duration = Duration::from_secs(30);
/// Defines polling interval used while waiting for API readiness.
const API_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
/// Defines graceful shutdown timeout for the orchestrator task.
const ORCHESTRATOR_SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(300);
/// Defines socket address used to probe API readiness.
const API_READY_ADDR: &str = "127.0.0.1:8000";
/// Defines cargo target directory used for docs builds.
const DOCS_TARGET_DIR: &str = "target/docs";
/// Defines output directory served by the docs HTTP server.
const DOCS_SERVE_DIR: &str = "target/docs/doc";
/// Defines isolated cargo home used during docs builds.
const DOCS_CARGO_HOME: &str = "target/.cargo-docs-home";
/// Defines environment variables injected into docs build commands.
const DOCS_BUILD_ENVS: [(&str, &str); 2] = [
    ("CARGO_TARGET_DIR", DOCS_TARGET_DIR),
    ("CARGO_HOME", DOCS_CARGO_HOME),
];
/// Defines cargo arguments used to generate workspace documentation.
const DOCS_ARGS: [&str; 14] = [
    "doc",
    "-p",
    "gig-log-api",
    "-p",
    "gig-log-common",
    "-p",
    "gig-log-dev-tools",
    "-p",
    "gig-log-frontend",
    "--no-deps",
    "--document-private-items",
    "--color",
    "always",
    "--locked",
];

/// Represents lifecycle events parsed from trunk output.
#[derive(Debug, Clone, Copy)]
enum TrunkEvent {
    /// Indicates a trunk build has started.
    BuildStarted,
    /// Indicates a trunk build finished successfully.
    BuildSucceeded,
    /// Indicates a trunk build failed.
    BuildFailed,
}

/// Stores drained trunk event state before waiting for fresh terminal events.
#[derive(Debug, Clone, Copy, Default)]
struct DrainedTrunkState {
    /// Tracks whether a start event was observed without terminal outcome.
    waiting_for_terminal: bool,
    /// Stores the terminal outcome observed after the latest start event.
    terminal_after_start: Option<TrunkEvent>,
}

/// Runs the development orchestrator entry workflow.
///
/// Validates local prerequisites, launches orchestrator services, and starts
/// the interactive terminal UI.
///
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if requirements are missing or runtime tasks fail.
pub async fn run() -> Result<()> {
    check_requirements()?;

    let (tui_tx, tui_rx) = mpsc::channel::<TuiEvent>(1024);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut orchestrator = tokio::spawn(run_orchestrator(tui_tx, shutdown_rx));
    let tui_result = tui::run_tui(tui_rx).await;

    let _ = shutdown_tx.send(true);
    if tokio::time::timeout(ORCHESTRATOR_SHUTDOWN_TIMEOUT, &mut orchestrator)
        .await
        .is_err()
    {
        orchestrator.abort();
        let _ = orchestrator.await;
    }

    tui_result
}

/// Runs the background orchestrator task until shutdown is requested.
///
/// # Arguments
///
/// * `tui_tx` — Sender used to publish TUI events.
/// * `shutdown_rx` — Watch channel receiver signaling orchestrator shutdown.
///
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if docs prerequisites, watchers, or service
/// process startup fails.
async fn run_orchestrator(
    tui_tx: mpsc::Sender<TuiEvent>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let (trunk_event_tx, mut trunk_event_rx) = mpsc::unbounded_channel::<TrunkEvent>();
    let mut processes = Vec::new();
    let mut log_senders = Vec::new();

    for service in [
        Service::Api,
        Service::Web,
        Service::Common,
        Service::DevTools,
        Service::Docs,
        Service::System,
    ] {
        log_senders.push((service, spawn_log_forwarder(&tui_tx)));
    }

    let system_log_tx = match log_senders
        .iter()
        .find(|(service, _)| *service == Service::System)
        .map(|(_, tx)| tx.clone())
    {
        Some(tx) => tx,
        None => return Ok(()),
    };

    let mut web_log_relay = match log_senders
        .iter()
        .find(|(service, _)| *service == Service::Web)
        .map(|(_, tx)| tx.clone())
    {
        Some(web_log_tx) => match web_log_relay::start(web_log_tx).await {
            Ok(relay) => {
                system_log(
                    &system_log_tx,
                    format!(
                        "Web log relay active at {} (proxy path {})",
                        web_log_relay::WEB_LOG_RELAY_ADDR,
                        web_log_relay::WEB_LOG_RELAY_PROXY_PATH
                    ),
                )
                .await;
                Some(relay)
            }
            Err(error) => {
                system_log(
                    &system_log_tx,
                    format!("Failed to start web log relay: {error}"),
                )
                .await;
                None
            }
        },
        None => None,
    };

    let _miniserve = start_docs_prerequisites().await?;
    if let Some(docs_tx) = log_senders
        .iter()
        .find(|(service, _)| *service == Service::Docs)
        .map(|(_, tx)| tx.clone())
    {
        let _ = docs_tx
            .send(LogEntry {
                service: Service::Docs,
                line: "Docs server running at http://localhost:7007 (port 7007)".to_string(),
            })
            .await;
    }

    for service in [Service::Api, Service::Web] {
        let (proc, mut log_rx) = ServiceProcess::spawn(service)?;
        processes.push(proc);
        let _ = tui_tx.send(TuiEvent::ServiceStarted(service)).await;

        let tx = match log_senders
            .iter()
            .find(|(item_service, _)| *item_service == service)
            .map(|(_, tx)| tx.clone())
        {
            Some(tx) => tx,
            None => continue,
        };

        let trunk_tx = trunk_event_tx.clone();
        tokio::spawn(async move {
            while let Some(entry) = log_rx.recv().await {
                if entry.service == Service::Web {
                    if let Some(event) = parse_trunk_event(&entry.line) {
                        let _ = trunk_tx.send(event);
                    }
                }

                if tx.send(entry).await.is_err() {
                    break;
                }
            }
        });
    }

    run_initial_docs_after_startup(&tui_tx, &system_log_tx, &log_senders, &mut trunk_event_rx)
        .await;

    let mut watch_stream = watcher::start(&WATCH_PATHS)?;

    system_log(
        &system_log_tx,
        "Watcher started for web/, api/, common/, dev-tools/".to_string(),
    )
    .await;

    loop {
        let first_intent = tokio::select! {
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    break;
                }
                continue;
            }
            item = watch_stream.recv() => item,
        };

        let Some(first_intent) = first_intent else {
            break;
        };

        let batch = watch_stream
            .collect_debounced_batch(first_intent, DEBOUNCE_DURATION)
            .await
            .merged_for_execution();

        if batch.is_empty() {
            continue;
        }

        let _ = tui_tx.send(TuiEvent::ClearLogs).await;

        system_log(
            &system_log_tx,
            format!("Executing batch: {}", describe_batch(batch)),
        )
        .await;

        execute_batch(
            batch,
            &tui_tx,
            &system_log_tx,
            &log_senders,
            &mut processes,
            &mut trunk_event_rx,
        )
        .await;
    }

    for mut proc in processes {
        proc.shutdown_fast().await;
    }

    if let Some(relay) = web_log_relay.take() {
        relay.shutdown().await;
    }

    Ok(())
}

/// Waits for initial web build completion and performs first docs build.
///
/// # Arguments
///
/// * `tui_tx` — Sender used to publish service lifecycle events.
/// * `system_tx` — Sender used to emit system log messages.
/// * `log_senders` — Per-service log sender map.
/// * `trunk_events` — Receiver for parsed trunk build lifecycle events.
async fn run_initial_docs_after_startup(
    tui_tx: &mpsc::Sender<TuiEvent>,
    system_tx: &mpsc::Sender<LogEntry>,
    log_senders: &[(Service, mpsc::Sender<LogEntry>)],
    trunk_events: &mut mpsc::UnboundedReceiver<TrunkEvent>,
) {
    system_log(
        system_tx,
        "Waiting for initial web build before first docs generation...".to_string(),
    )
    .await;

    let drained = drain_trunk_events(trunk_events);

    match wait_for_web_build(trunk_events, drained).await {
        Ok(()) => {
            system_log(
                system_tx,
                "Initial web build completed. Generating docs...".to_string(),
            )
            .await;

            if let Err(error) = reset_docs_output_dir().await {
                system_log(
                    system_tx,
                    format!("Skipping initial docs generation: {error}"),
                )
                .await;
                return;
            }

            let docs_ok = run_build_step(
                Service::Docs,
                "cargo",
                &DOCS_ARGS,
                None,
                Some(&DOCS_BUILD_ENVS),
                tui_tx,
                log_senders,
            )
            .await;

            if docs_ok {
                let _ = run_build_step(
                    Service::Docs,
                    "cargo",
                    &["run", "-p", "gig-log-dev-tools", "--", "docs-index"],
                    None,
                    Some(&DOCS_BUILD_ENVS),
                    tui_tx,
                    log_senders,
                )
                .await;
            }
        }
        Err(error) => {
            system_log(
                system_tx,
                format!("Skipping initial docs generation: {error}"),
            )
            .await;
        }
    }
}

/// Executes rebuild and restart actions for a merged watch-event batch.
///
/// # Arguments
///
/// * `batch` — Merged intent batch to execute.
/// * `tui_tx` — Sender used to publish service lifecycle events.
/// * `system_tx` — Sender used to emit system log messages.
/// * `log_senders` — Per-service log sender map.
/// * `processes` — Mutable list of long-running orchestrated processes.
/// * `trunk_events` — Receiver for parsed trunk build lifecycle events.
async fn execute_batch(
    batch: IntentBatch,
    tui_tx: &mpsc::Sender<TuiEvent>,
    system_tx: &mpsc::Sender<LogEntry>,
    log_senders: &[(Service, mpsc::Sender<LogEntry>)],
    processes: &mut Vec<ServiceProcess>,
    trunk_events: &mut mpsc::UnboundedReceiver<TrunkEvent>,
) {
    let mut docs_needed = false;
    let mut docs_after_api_restart = false;

    if batch.common {
        if run_build_step(
            Service::Common,
            "cargo",
            &["build", "-p", "gig-log-common"],
            None,
            None,
            tui_tx,
            log_senders,
        )
        .await
        {
            if restart_api(processes, tui_tx, system_tx, log_senders).await {
                docs_needed = true;
                docs_after_api_restart = true;
            }
        }
    }

    if batch.api {
        if restart_api(processes, tui_tx, system_tx, log_senders).await {
            docs_needed = true;
            docs_after_api_restart = true;
        }
    }

    if batch.web {
        let drained = drain_trunk_events(trunk_events);
        match wait_for_web_build(trunk_events, drained).await {
            Ok(()) => {
                system_log(system_tx, "Web build completed via trunk.".to_string()).await;
                docs_needed = true;
            }
            Err(error) => {
                system_log(
                    system_tx,
                    format!("Skipping docs after web change: {error}"),
                )
                .await;
            }
        }
    }

    if batch.dev_tools
        && run_build_step(
            Service::DevTools,
            "cargo",
            &["build", "-p", "gig-log-dev-tools"],
            None,
            None,
            tui_tx,
            log_senders,
        )
        .await
    {
        docs_needed = true;
    }

    if docs_needed {
        if docs_after_api_restart {
            match wait_for_api_ready().await {
                Ok(()) => {
                    system_log(
                        system_tx,
                        "API is ready. Running docs generation...".to_string(),
                    )
                    .await;
                }
                Err(error) => {
                    system_log(
                        system_tx,
                        format!("Skipping docs after API restart: {error}"),
                    )
                    .await;
                    return;
                }
            }
        }

        if let Err(error) = reset_docs_output_dir().await {
            system_log(system_tx, format!("Skipping docs generation: {error}")).await;
            return;
        }

        let _ = run_build_step(
            Service::Docs,
            "cargo",
            &DOCS_ARGS,
            None,
            Some(&DOCS_BUILD_ENVS),
            tui_tx,
            log_senders,
        )
        .await;

        let _ = run_build_step(
            Service::Docs,
            "cargo",
            &["run", "-p", "gig-log-dev-tools", "--", "docs-index"],
            None,
            Some(&DOCS_BUILD_ENVS),
            tui_tx,
            log_senders,
        )
        .await;
    }
}

/// Restarts the API process and reconnects log forwarding.
///
/// # Arguments
///
/// * `processes` — Mutable list of running long-lived service processes.
/// * `tui_tx` — Sender used to publish service lifecycle events.
/// * `system_tx` — Sender used to emit system log messages.
/// * `log_senders` — Per-service log sender map.
///
/// # Returns
///
/// A boolean indicating whether API restart succeeded.
async fn restart_api(
    processes: &mut Vec<ServiceProcess>,
    tui_tx: &mpsc::Sender<TuiEvent>,
    system_tx: &mpsc::Sender<LogEntry>,
    log_senders: &[(Service, mpsc::Sender<LogEntry>)],
) -> bool {
    system_log(system_tx, "Restarting API process...".to_string()).await;

    if let Some(index) = processes
        .iter()
        .position(|proc| proc.service == Service::Api)
    {
        let mut existing = processes.remove(index);
        existing.shutdown().await;
        let _ = tui_tx.send(TuiEvent::ServiceExited(Service::Api)).await;
    }

    match ServiceProcess::spawn(Service::Api) {
        Ok((proc, mut log_rx)) => {
            let _ = tui_tx.send(TuiEvent::ServiceStarted(Service::Api)).await;
            processes.push(proc);

            let tx = tui_tx.clone();
            let log_tx = log_senders
                .iter()
                .find(|(service, _)| *service == Service::Api)
                .map(|(_, sender)| sender.clone());

            tokio::spawn(async move {
                while let Some(entry) = log_rx.recv().await {
                    if let Some(log_tx) = &log_tx {
                        let _ = log_tx.send(entry).await;
                    } else if tx.send(TuiEvent::Log(entry)).await.is_err() {
                        break;
                    }
                }
                let _ = tx.send(TuiEvent::ServiceExited(Service::Api)).await;
            });

            true
        }
        Err(error) => {
            system_log(system_tx, format!("Failed to restart API process: {error}")).await;
            false
        }
    }
}

/// Runs a single build step while emitting lifecycle and log events.
///
/// # Arguments
///
/// * `service` — Service channel associated with this build step.
/// * `cmd` — Executable name to run.
/// * `args` — Command-line arguments passed to `cmd`.
/// * `working_dir` — Optional working directory override.
/// * `envs` — Optional environment variables to inject.
/// * `tui_tx` — Sender used to emit service started/exited events.
/// * `log_senders` — Per-service log sender map.
///
/// # Returns
///
/// A boolean indicating whether the command completed successfully.
async fn run_build_step(
    service: Service,
    cmd: &str,
    args: &[&str],
    working_dir: Option<&str>,
    envs: Option<&[(&str, &str)]>,
    tui_tx: &mpsc::Sender<TuiEvent>,
    log_senders: &[(Service, mpsc::Sender<LogEntry>)],
) -> bool {
    let log_tx = match log_senders
        .iter()
        .find(|(item_service, _)| *item_service == service)
    {
        Some((_, tx)) => tx,
        None => return false,
    };

    let _ = tui_tx.send(TuiEvent::ServiceStarted(service)).await;
    let result = run_job(service, cmd, args, working_dir, envs, log_tx).await;
    let _ = tui_tx.send(TuiEvent::ServiceExited(service)).await;

    match result {
        Ok(success) => success,
        Err(error) => {
            let _ = log_tx
                .send(LogEntry {
                    service,
                    line: format!("Command failed: {error}"),
                })
                .await;
            false
        }
    }
}

/// Waits until the API port is reachable.
///
/// # Returns
///
/// An empty [`Result`] once the API socket accepts connections.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if readiness polling exceeds timeout.
async fn wait_for_api_ready() -> Result<()> {
    let fut = async {
        loop {
            if TcpStream::connect(API_READY_ADDR).await.is_ok() {
                break;
            }
            tokio::time::sleep(API_READY_POLL_INTERVAL).await;
        }

        Ok::<(), anyhow::Error>(())
    };

    tokio::time::timeout(API_READY_TIMEOUT, fut)
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for API to bind to {API_READY_ADDR}"))??;

    Ok(())
}

/// Waits for a trunk build start event followed by a terminal outcome.
///
/// # Arguments
///
/// * `rx` — Receiver of parsed trunk build lifecycle events.
/// * `drained` — State collected from events already queued before waiting.
///
/// # Returns
///
/// An empty [`Result`] when a successful build terminal event is observed.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if trunk reports a failure, times out, or the
/// event stream ends before success.
async fn wait_for_web_build(
    rx: &mut mpsc::UnboundedReceiver<TrunkEvent>,
    drained: DrainedTrunkState,
) -> Result<()> {
    match drained.terminal_after_start {
        Some(TrunkEvent::BuildFailed) => anyhow::bail!("trunk reported build failure"),
        Some(TrunkEvent::BuildSucceeded) => return Ok(()),
        _ => {}
    }

    let mut seen_started = drained.waiting_for_terminal;

    let fut = async {
        while let Some(event) = rx.recv().await {
            match event {
                TrunkEvent::BuildStarted => {
                    seen_started = true;
                }
                TrunkEvent::BuildSucceeded => {
                    if seen_started {
                        return Ok(());
                    }
                }
                TrunkEvent::BuildFailed => {
                    if seen_started {
                        anyhow::bail!("trunk reported build failure");
                    }
                }
            }
        }
        anyhow::bail!("trunk event stream ended")
    };

    tokio::time::timeout(WEB_BUILD_TIMEOUT, fut)
        .await
        .map_err(|_| anyhow::anyhow!("timed out waiting for trunk build"))??;

    Ok(())
}

/// Drains pending trunk events and captures the latest terminal context.
///
/// # Arguments
///
/// * `rx` — Receiver to drain without awaiting.
///
/// # Returns
///
/// A [`DrainedTrunkState`] snapshot used by subsequent wait logic.
fn drain_trunk_events(rx: &mut mpsc::UnboundedReceiver<TrunkEvent>) -> DrainedTrunkState {
    let mut state = DrainedTrunkState::default();
    while let Ok(event) = rx.try_recv() {
        match event {
            TrunkEvent::BuildStarted => {
                state.waiting_for_terminal = true;
                state.terminal_after_start = None;
            }
            TrunkEvent::BuildSucceeded => {
                if state.waiting_for_terminal {
                    state.waiting_for_terminal = false;
                    state.terminal_after_start = Some(TrunkEvent::BuildSucceeded);
                }
            }
            TrunkEvent::BuildFailed => {
                if state.waiting_for_terminal {
                    state.waiting_for_terminal = false;
                    state.terminal_after_start = Some(TrunkEvent::BuildFailed);
                }
            }
        }
    }
    state
}

/// Parses a trunk output line into a lifecycle event when recognized.
///
/// # Arguments
///
/// * `line` — Raw web service log line.
///
/// # Returns
///
/// An optional [`TrunkEvent`] when the line contains build lifecycle markers.
fn parse_trunk_event(line: &str) -> Option<TrunkEvent> {
    let lower = line.to_lowercase();

    if lower.contains("starting build") || lower.contains("building") {
        return Some(TrunkEvent::BuildStarted);
    }

    if lower.contains('❌')
        || lower.contains("build failed")
        || lower.contains("failed build")
        || (lower.contains("error") && lower.contains("build"))
    {
        return Some(TrunkEvent::BuildFailed);
    }

    if lower.contains('✅')
        || lower.contains("build succeeded")
        || (lower.contains("finished") && (lower.contains("build") || lower.contains("release")))
    {
        return Some(TrunkEvent::BuildSucceeded);
    }

    None
}

/// Spawns a background task that forwards log entries into TUI events.
///
/// # Arguments
///
/// * `tui_tx` — Sender receiving forwarded [`TuiEvent::Log`] events.
///
/// # Returns
///
/// A [`mpsc::Sender`] used by producers to submit [`LogEntry`] values.
fn spawn_log_forwarder(tui_tx: &mpsc::Sender<TuiEvent>) -> mpsc::Sender<LogEntry> {
    let (log_tx, mut log_rx) = mpsc::channel::<LogEntry>(256);
    let tx = tui_tx.clone();

    tokio::spawn(async move {
        while let Some(entry) = log_rx.recv().await {
            if tx.send(TuiEvent::Log(entry)).await.is_err() {
                break;
            }
        }
    });

    log_tx
}

/// Emits a system-scoped log line.
///
/// # Arguments
///
/// * `tx` — Log sender for system service entries.
/// * `message` — Message content to emit.
async fn system_log(tx: &mpsc::Sender<LogEntry>, message: String) {
    let _ = tx
        .send(LogEntry {
            service: Service::System,
            line: message,
        })
        .await;
}

/// Describes a merged intent batch for human-readable logging.
///
/// # Arguments
///
/// * `batch` — Intent batch to describe.
///
/// # Returns
///
/// A comma-separated label string for enabled intents.
fn describe_batch(batch: IntentBatch) -> String {
    let mut labels = Vec::new();
    if batch.common {
        labels.push("common");
    }
    if batch.api {
        labels.push("api");
    }
    if batch.web {
        labels.push("web");
    }
    if batch.dev_tools {
        labels.push("dev-tools");
    }
    labels.join(", ")
}

/// Starts docs-serving prerequisites for orchestrator-managed docs builds.
///
/// # Returns
///
/// A running `miniserve` child process handle.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if docs directory creation or `miniserve`
/// startup fails.
async fn start_docs_prerequisites() -> Result<tokio::process::Child> {
    use std::process::Stdio;

    let _ = Command::new("sh")
        .args(["-c", "lsof -ti :7007 | xargs -r kill 2>/dev/null"])
        .status()
        .await;

    tokio::fs::create_dir_all(DOCS_SERVE_DIR).await?;

    let child = Command::new("miniserve")
        .args(["--index", "index.html", "-p", "7007", DOCS_SERVE_DIR])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()?;

    Ok(child)
}

/// Recreates docs output directory before each docs generation pass.
///
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if directory removal or creation fails.
async fn reset_docs_output_dir() -> Result<()> {
    if tokio::fs::try_exists(DOCS_SERVE_DIR).await? {
        tokio::fs::remove_dir_all(DOCS_SERVE_DIR).await?;
    }
    tokio::fs::create_dir_all(DOCS_SERVE_DIR).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TrunkEvent, drain_trunk_events, parse_trunk_event, wait_for_web_build};
    use tokio::sync::mpsc;

    #[test]
    fn parses_trunk_success_line() {
        let line = "2026-03-12T07:47:41.435056Z  INFO ✅ success";
        assert!(matches!(
            parse_trunk_event(line),
            Some(TrunkEvent::BuildSucceeded)
        ));
    }

    #[test]
    fn parses_trunk_started_line() {
        let line = "2026-03-12T07:47:40.494783Z  INFO 📦 starting build";
        assert!(matches!(
            parse_trunk_event(line),
            Some(TrunkEvent::BuildStarted)
        ));
    }

    #[test]
    fn parses_trunk_failure_line() {
        let line = "error during build";
        assert!(matches!(
            parse_trunk_event(line),
            Some(TrunkEvent::BuildFailed)
        ));
    }

    #[tokio::test]
    async fn wait_ignores_success_before_start() {
        let (tx, mut rx) = mpsc::unbounded_channel();

        let _ = tx.send(TrunkEvent::BuildSucceeded);
        let _ = tx.send(TrunkEvent::BuildStarted);
        let _ = tx.send(TrunkEvent::BuildSucceeded);

        let drained = drain_trunk_events(&mut rx);
        assert!(wait_for_web_build(&mut rx, drained).await.is_ok());
    }
}
