use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::process::Command;

mod doc_index;

const DOCS_TARGET_DIR: &str = "target/docs";
const DOCS_SERVE_DIR: &str = "target/docs/doc";
const DOCS_CARGO_HOME: &str = "target/.cargo-docs-home";
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

/// Builds workspace documentation, starts a live-reload docs server, and
/// watches for source changes.
///
/// # Errors
///
/// Returns an error if required tools (`cargo-watch`, `miniserve`) are
/// not installed, the documentation build fails, or the server cannot
/// start.
pub async fn run() -> Result<()> {
    check_requirements().await?;

    // Kill any leftover doc server on port 7007
    let _ = Command::new("sh")
        .args(["-c", "lsof -ti :7007 | xargs -r kill 2>/dev/null"])
        .status()
        .await;

    reset_docs_output_dir().await?;

    // Initial cargo doc build
    println!("Building workspace documentation...");
    let status = Command::new("cargo")
        .args(DOCS_ARGS)
        .env("CARGO_TARGET_DIR", DOCS_TARGET_DIR)
        .env("CARGO_HOME", DOCS_CARGO_HOME)
        .status()
        .await
        .context("Failed to run cargo doc")?;

    if !status.success() {
        eprintln!("Warning: cargo doc exited with non-zero status, continuing anyway...");
    }

    // Generate doc index
    doc_index::generate(DOCS_TARGET_DIR)?;

    // Start miniserve in background
    let mut miniserve = Command::new("miniserve")
        .args(["--index", "index.html", "-p", "7007", DOCS_SERVE_DIR])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .context("Failed to start miniserve")?;

    println!("Docs server running at http://localhost:7007 (port 7007)");

    // Run cargo watch to rebuild docs on changes
    let mut watch = Command::new("cargo")
        .args([
            "watch",
            "-s",
            "rm -rf target/docs/doc && mkdir -p target/docs/doc && CARGO_TARGET_DIR=target/docs CARGO_HOME=target/.cargo-docs-home cargo doc -p gig-log-api -p gig-log-common -p gig-log-dev-tools -p gig-log-frontend --no-deps --document-private-items --color always --locked && CARGO_TARGET_DIR=target/docs CARGO_HOME=target/.cargo-docs-home cargo run -p gig-log-dev-tools -- docs-index",
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

/// Removes and recreates the docs output directory to ensure a clean build.
///
/// # Errors
///
/// Returns an error if the directory cannot be removed or recreated.
async fn reset_docs_output_dir() -> Result<()> {
    if tokio::fs::try_exists(DOCS_SERVE_DIR).await? {
        tokio::fs::remove_dir_all(DOCS_SERVE_DIR).await?;
    }
    tokio::fs::create_dir_all(DOCS_SERVE_DIR).await?;
    Ok(())
}

/// Generates the workspace-level doc index page.
///
/// Delegates to [`doc_index::generate`] with the default docs target
/// directory.
///
/// # Errors
///
/// Returns an error if index generation fails.
pub fn generate_index() -> Result<()> {
    doc_index::generate(DOCS_TARGET_DIR)
}

/// Verifies that `cargo-watch` and `miniserve` are installed before starting the docs server.
///
/// # Errors
///
/// Returns an error if either `cargo-watch` or `miniserve` is not
/// found on the system PATH.
async fn check_requirements() -> Result<()> {
    let cargo_watch = Command::new("which")
        .arg("cargo-watch")
        .stdout(Stdio::null())
        .status()
        .await;

    if cargo_watch.is_err() || !cargo_watch.unwrap().success() {
        anyhow::bail!("cargo-watch is not installed. Install it with: cargo install cargo-watch");
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
