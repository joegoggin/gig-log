use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use super::log_store::{LogEntry, Service};

pub struct ServiceProcess {
    pub service: Service,
    pub child: Child,
    process_group: u32,
}

impl ServiceProcess {
    pub fn spawn(service: Service) -> Result<(Self, mpsc::Receiver<LogEntry>)> {
        let (cmd, args, working_dir) = service_command(service)?;
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

    pub async fn shutdown(&mut self) {
        self.shutdown_with_timeout(Duration::from_secs(3)).await;
    }

    pub async fn shutdown_fast(&mut self) {
        self.shutdown_with_timeout(Duration::from_millis(200)).await;
    }

    async fn shutdown_with_timeout(&mut self, timeout: Duration) {
        self.signal_process_group(libc::SIGTERM);

        let wait_result = tokio::time::timeout(timeout, self.child.wait()).await;

        if wait_result.is_err() {
            self.signal_process_group(libc::SIGKILL);
            let _ = self.child.kill().await;
        }
    }

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

fn service_command(
    service: Service,
) -> Result<(&'static str, Vec<&'static str>, Option<&'static str>)> {
    match service {
        Service::Api => Ok(("cargo", vec!["run", "-p", "gig-log-api"], None)),
        Service::Web => Ok(("trunk", vec!["serve"], Some("web/"))),
        _ => anyhow::bail!("{} is not a long-running service", service.label()),
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
