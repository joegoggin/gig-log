mod log_store;
mod process;
mod tui;
mod ui;
mod watcher;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use anyhow::Result;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::sync::{mpsc, watch};

use log_store::{LogEntry, Service};
use process::{ServiceProcess, check_requirements, run_job};
use tui::TuiEvent;
use watcher::IntentBatch;

const WATCH_PATHS: [&str; 4] = ["web", "api", "common", "dev-tools"];
const DEBOUNCE_DURATION: Duration = Duration::from_millis(400);
const WEB_BUILD_TIMEOUT: Duration = Duration::from_secs(60);
const API_READY_TIMEOUT: Duration = Duration::from_secs(30);
const API_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
const ORCHESTRATOR_SHUTDOWN_TIMEOUT: Duration = Duration::from_millis(300);
const API_READY_ADDR: &str = "127.0.0.1:8000";
const DOCS_TARGET_DIR: &str = "target/docs";
const DOCS_SERVE_DIR: &str = "target/docs/doc";
const DOCS_CARGO_HOME: &str = "target/.cargo-docs-home";
const DOCS_BUILD_ENVS: [(&str, &str); 2] = [
    ("CARGO_TARGET_DIR", DOCS_TARGET_DIR),
    ("CARGO_HOME", DOCS_CARGO_HOME),
];
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

#[derive(Debug, Clone, Copy)]
enum TrunkEvent {
    BuildStarted,
    BuildSucceeded,
    BuildFailed,
}

#[derive(Default)]
struct TrunkState {
    success_count: AtomicU64,
    fail_count: AtomicU64,
}

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

async fn run_orchestrator(
    tui_tx: mpsc::Sender<TuiEvent>,
    mut shutdown_rx: watch::Receiver<bool>,
) -> Result<()> {
    let (trunk_event_tx, mut trunk_event_rx) = mpsc::unbounded_channel::<TrunkEvent>();
    let trunk_state = Arc::new(TrunkState::default());

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
        let trunk_state = trunk_state.clone();

        tokio::spawn(async move {
            while let Some(entry) = log_rx.recv().await {
                if entry.service == Service::Web {
                    if let Some(event) = parse_trunk_event(&entry.line) {
                        match event {
                            TrunkEvent::BuildSucceeded => {
                                trunk_state.success_count.fetch_add(1, Ordering::SeqCst);
                            }
                            TrunkEvent::BuildFailed => {
                                trunk_state.fail_count.fetch_add(1, Ordering::SeqCst);
                            }
                            TrunkEvent::BuildStarted => {}
                        }
                        let _ = trunk_tx.send(event);
                    }
                }

                if tx.send(entry).await.is_err() {
                    break;
                }
            }
        });
    }

    let system_log_tx = match log_senders
        .iter()
        .find(|(service, _)| *service == Service::System)
        .map(|(_, tx)| tx.clone())
    {
        Some(tx) => tx,
        None => return Ok(()),
    };

    run_initial_docs_after_startup(
        &tui_tx,
        &system_log_tx,
        &log_senders,
        &mut trunk_event_rx,
        &trunk_state,
    )
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

        let web_baseline = (
            trunk_state.success_count.load(Ordering::SeqCst),
            trunk_state.fail_count.load(Ordering::SeqCst),
        );

        let batch = watch_stream
            .collect_debounced_batch(first_intent, DEBOUNCE_DURATION)
            .await
            .merged_for_execution();

        if batch.is_empty() {
            continue;
        }

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
            web_baseline,
        )
        .await;
    }

    for mut proc in processes {
        proc.shutdown_fast().await;
    }

    Ok(())
}

async fn run_initial_docs_after_startup(
    tui_tx: &mpsc::Sender<TuiEvent>,
    system_tx: &mpsc::Sender<LogEntry>,
    log_senders: &[(Service, mpsc::Sender<LogEntry>)],
    trunk_events: &mut mpsc::UnboundedReceiver<TrunkEvent>,
    trunk_state: &TrunkState,
) {
    system_log(
        system_tx,
        "Waiting for initial web build before first docs generation...".to_string(),
    )
    .await;

    let baseline = (
        trunk_state.success_count.load(Ordering::SeqCst),
        trunk_state.fail_count.load(Ordering::SeqCst),
    );

    match wait_for_web_build(trunk_events, baseline).await {
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

async fn execute_batch(
    batch: IntentBatch,
    tui_tx: &mpsc::Sender<TuiEvent>,
    system_tx: &mpsc::Sender<LogEntry>,
    log_senders: &[(Service, mpsc::Sender<LogEntry>)],
    processes: &mut Vec<ServiceProcess>,
    trunk_events: &mut mpsc::UnboundedReceiver<TrunkEvent>,
    web_baseline: (u64, u64),
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
        match wait_for_web_build(trunk_events, web_baseline).await {
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
                    system_log(system_tx, "API is ready. Running docs generation...".to_string())
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

async fn wait_for_web_build(
    rx: &mut mpsc::UnboundedReceiver<TrunkEvent>,
    baseline: (u64, u64),
) -> Result<()> {
    let (baseline_success, baseline_fail) = baseline;
    let mut success_count = baseline_success;
    let mut fail_count = baseline_fail;
    let mut seen_started = false;

    let fut = async {
        while let Some(event) = rx.recv().await {
            match event {
                TrunkEvent::BuildStarted => {
                    seen_started = true;
                }
                TrunkEvent::BuildSucceeded => {
                    success_count += 1;
                    if seen_started && success_count > baseline_success {
                        return Ok(());
                    }
                }
                TrunkEvent::BuildFailed => {
                    fail_count += 1;
                    if seen_started && fail_count > baseline_fail {
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

    if lower.contains("finished") && (lower.contains("build") || lower.contains("release")) {
        return Some(TrunkEvent::BuildSucceeded);
    }

    if lower.contains('✅') || lower.contains("success") {
        return Some(TrunkEvent::BuildSucceeded);
    }

    None
}

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

async fn system_log(tx: &mpsc::Sender<LogEntry>, message: String) {
    let _ = tx
        .send(LogEntry {
            service: Service::System,
            line: message,
        })
        .await;
}

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

async fn reset_docs_output_dir() -> Result<()> {
    if tokio::fs::try_exists(DOCS_SERVE_DIR).await? {
        tokio::fs::remove_dir_all(DOCS_SERVE_DIR).await?;
    }
    tokio::fs::create_dir_all(DOCS_SERVE_DIR).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{TrunkEvent, parse_trunk_event, wait_for_web_build};
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

        assert!(wait_for_web_build(&mut rx, (0, 0)).await.is_ok());
    }
}
