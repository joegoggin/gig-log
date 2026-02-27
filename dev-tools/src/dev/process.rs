use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use super::log_store::{LogEntry, Service};

pub struct ServiceProcess {
    #[allow(dead_code)]
    pub service: Service,
    pub child: Child,
    process_group: u32,
}

impl ServiceProcess {
    pub fn spawn(service: Service) -> Result<(Self, mpsc::Receiver<LogEntry>)> {
        let (cmd, args, working_dir) = service_command(service);
        let (tx, rx) = mpsc::channel(256);

        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(dir) = working_dir {
            command.current_dir(dir);
        }

        command.env("CARGO_TERM_COLOR", "always");
        command.env("CLICOLOR_FORCE", "1");

        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
        command.kill_on_drop(true);

        // Create a new process group so we can kill all children
        unsafe {
            command.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let mut child = command.spawn().with_context(|| {
            format!("Failed to spawn {} service", service.label())
        })?;

        let pid = child.id().unwrap_or(0);

        // Spawn stdout reader
        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            tokio::spawn(read_lines(stdout, service, tx));
        }

        // Spawn stderr reader
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

    pub async fn shutdown(&mut self) {
        // Send SIGTERM to the entire process group
        if self.process_group > 0 {
            unsafe {
                libc::kill(-(self.process_group as i32), libc::SIGTERM);
            }
        }

        // Wait up to 3 seconds for graceful shutdown
        let timeout =
            tokio::time::timeout(std::time::Duration::from_secs(3), self.child.wait()).await;

        if timeout.is_err() {
            // Force kill the process group
            if self.process_group > 0 {
                unsafe {
                    libc::kill(-(self.process_group as i32), libc::SIGKILL);
                }
            }
            let _ = self.child.kill().await;
        }
    }
}

fn service_command(service: Service) -> (&'static str, Vec<&'static str>, Option<&'static str>) {
    match service {
        Service::Api => (
            "cargo",
            vec!["watch", "-x", "run -p gig-log-api"],
            None,
        ),
        Service::Web => ("trunk", vec!["serve"], Some("web/")),
        Service::Docs => (
            "cargo",
            vec![
                "watch",
                "-s",
                "cargo doc --workspace --document-private-items --color always && bash scripts/generate-doc-index.sh",
            ],
            None,
        ),
    }
}

async fn read_lines<R: tokio::io::AsyncRead + Unpin>(
    reader: R,
    service: Service,
    tx: mpsc::Sender<LogEntry>,
) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if tx.send(LogEntry { service, line }).await.is_err() {
            break;
        }
    }
}

pub fn check_requirements(services: &[Service]) -> Result<()> {
    for service in services {
        match service {
            Service::Api => {
                if std::process::Command::new("which")
                    .arg("cargo-watch")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| !s.success())
                    .unwrap_or(true)
                {
                    anyhow::bail!(
                        "cargo-watch is not installed. Install it with: cargo install cargo-watch"
                    );
                }
            }
            Service::Web => {
                if std::process::Command::new("which")
                    .arg("trunk")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| !s.success())
                    .unwrap_or(true)
                {
                    anyhow::bail!(
                        "trunk is not installed. Install it with: cargo install trunk"
                    );
                }
            }
            Service::Docs => {
                if std::process::Command::new("which")
                    .arg("cargo-watch")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| !s.success())
                    .unwrap_or(true)
                {
                    anyhow::bail!(
                        "cargo-watch is not installed. Install it with: cargo install cargo-watch"
                    );
                }
                if std::process::Command::new("which")
                    .arg("miniserve")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|s| !s.success())
                    .unwrap_or(true)
                {
                    anyhow::bail!(
                        "miniserve is not installed. Install it with: cargo install miniserve"
                    );
                }
            }
        }
    }
    Ok(())
}
