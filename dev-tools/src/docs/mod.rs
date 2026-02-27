use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::process::Command;

pub async fn run() -> Result<()> {
    check_requirements().await?;

    // Kill any leftover doc server on port 7007
    let _ = Command::new("sh")
        .args(["-c", "lsof -ti :7007 | xargs -r kill 2>/dev/null"])
        .status()
        .await;

    // Ensure target/doc exists
    tokio::fs::create_dir_all("target/doc").await?;

    // Initial cargo doc build
    println!("Building workspace documentation...");
    let status = Command::new("cargo")
        .args([
            "doc",
            "--workspace",
            "--document-private-items",
            "--color",
            "always",
        ])
        .status()
        .await
        .context("Failed to run cargo doc")?;

    if !status.success() {
        eprintln!("Warning: cargo doc exited with non-zero status, continuing anyway...");
    }

    // Generate doc index
    let status = Command::new("bash")
        .args(["scripts/generate-doc-index.sh"])
        .status()
        .await
        .context("Failed to run generate-doc-index.sh")?;

    if !status.success() {
        anyhow::bail!("generate-doc-index.sh failed");
    }

    // Start miniserve in background
    let mut miniserve = Command::new("miniserve")
        .args(["--index", "index.html", "-p", "7007", "target/doc"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("Failed to start miniserve")?;

    println!("Docs available at http://localhost:7007");

    // Run cargo watch to rebuild docs on changes
    let mut watch = Command::new("cargo")
        .args([
            "watch",
            "-s",
            "cargo doc --workspace --document-private-items --color always && bash scripts/generate-doc-index.sh",
        ])
        .kill_on_drop(true)
        .spawn()
        .context("Failed to start cargo watch")?;

    // Wait for either process to exit
    tokio::select! {
        result = watch.wait() => {
            let status = result.context("cargo watch failed")?;
            if !status.success() {
                eprintln!("cargo watch exited with non-zero status");
            }
        }
        result = miniserve.wait() => {
            let status = result.context("miniserve failed")?;
            if !status.success() {
                eprintln!("miniserve exited unexpectedly");
            }
        }
    }

    Ok(())
}

async fn check_requirements() -> Result<()> {
    let cargo_watch = Command::new("which")
        .arg("cargo-watch")
        .stdout(Stdio::null())
        .status()
        .await;

    if cargo_watch.is_err() || !cargo_watch.unwrap().success() {
        anyhow::bail!(
            "cargo-watch is not installed. Install it with: cargo install cargo-watch"
        );
    }

    let miniserve = Command::new("which")
        .arg("miniserve")
        .stdout(Stdio::null())
        .status()
        .await;

    if miniserve.is_err() || !miniserve.unwrap().success() {
        anyhow::bail!("miniserve is not installed. Install it with: cargo install miniserve");
    }

    Ok(())
}
