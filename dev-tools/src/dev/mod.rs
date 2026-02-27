mod log_store;
mod process;
mod tui;
mod ui;

use anyhow::Result;
use tokio::sync::mpsc;

use log_store::Service;
use process::{ServiceProcess, check_requirements};
use tui::TuiEvent;

const ALL_SERVICES: [Service; 3] = [Service::Api, Service::Web, Service::Docs];

pub async fn run(service_names: Option<Vec<String>>) -> Result<()> {
    let services = match service_names {
        Some(names) => {
            let mut services = Vec::new();
            for name in &names {
                match name.to_lowercase().as_str() {
                    "api" => services.push(Service::Api),
                    "web" => services.push(Service::Web),
                    "docs" => services.push(Service::Docs),
                    other => anyhow::bail!("Unknown service: {other}. Valid services: api, web, docs"),
                }
            }
            services
        }
        None => ALL_SERVICES.to_vec(),
    };

    check_requirements(&services)?;

    // For the docs service, we need miniserve and the initial doc build
    if services.contains(&Service::Docs) {
        start_docs_prerequisites().await?;
    }

    let (tui_tx, tui_rx) = mpsc::channel::<TuiEvent>(512);

    // Spawn all service processes
    let mut processes = Vec::new();
    for &service in &services {
        let (proc, mut log_rx) = ServiceProcess::spawn(service)?;
        processes.push(proc);

        let tx = tui_tx.clone();
        let _ = tx.send(TuiEvent::ServiceStarted(service)).await;

        // Forward log entries to TUI
        tokio::spawn(async move {
            while let Some(entry) = log_rx.recv().await {
                if tx.send(TuiEvent::Log(entry)).await.is_err() {
                    break;
                }
            }
            let _ = tx.send(TuiEvent::ServiceExited(service)).await;
        });
    }

    // Drop our copy of the sender so TUI can detect when all senders are gone
    drop(tui_tx);

    // Run the TUI (blocks until user quits)
    let result = tui::run_tui(tui_rx).await;

    // Shutdown all processes
    for mut proc in processes {
        proc.shutdown().await;
    }

    result
}

async fn start_docs_prerequisites() -> Result<()> {
    use std::process::Stdio;
    use tokio::process::Command;

    // Kill any leftover doc server on port 7007
    let _ = Command::new("sh")
        .args(["-c", "lsof -ti :7007 | xargs -r kill 2>/dev/null"])
        .status()
        .await;

    // Ensure target/doc exists
    tokio::fs::create_dir_all("target/doc").await?;

    // Initial cargo doc build
    let _ = Command::new("cargo")
        .args([
            "doc",
            "--workspace",
            "--document-private-items",
            "--color",
            "always",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    // Generate doc index
    let _ = Command::new("bash")
        .args(["scripts/generate-doc-index.sh"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    // Start miniserve in background (kill_on_drop handles cleanup)
    let _miniserve = Command::new("miniserve")
        .args(["--index", "index.html", "-p", "7007", "target/doc"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn();

    Ok(())
}
