//! Builds and serves workspace rustdoc output for local development.
//!
//! This module powers the `docs` and `docs-index` subcommands. It validates
//! required tooling, generates rustdoc output for workspace crates (including
//! private items), writes a workspace landing page, serves docs over HTTP, and
//! watches for source changes to keep generated docs up to date.
//!
//! # Modules
//!
//! - [`doc_index`] — Workspace `index.html` generation for rustdoc output.

use std::process::Stdio;

use anyhow::{Context, Result};
use tokio::process::Command;

mod doc_index;

/// Defines the cargo target directory used for docs builds.
const DOCS_TARGET_DIR: &str = "target/docs";
/// Defines the directory served by `miniserve`.
const DOCS_SERVE_DIR: &str = "target/docs/doc";
/// Defines the isolated cargo home used during docs builds.
const DOCS_CARGO_HOME: &str = "target/.cargo-docs-home";
/// Defines rustdoc lint settings enforced during docs generation.
const DOCS_RUSTDOCFLAGS: &str = "-D rustdoc::broken_intra_doc_links";
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

/// Runs the docs workflow for local development.
///
/// Validates required tools, clears the docs output directory, builds
/// workspace docs, generates the workspace `index.html`, starts `miniserve` on
/// port `7007`, and launches `cargo watch` to rebuild docs on file changes.
///
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if requirements are missing, docs generation
/// fails, or background docs processes cannot be started.
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
        .env("RUSTDOCFLAGS", DOCS_RUSTDOCFLAGS)
        .status()
        .await
        .context("Failed to run cargo doc")?;

    if !status.success() {
        anyhow::bail!("cargo doc exited with non-zero status");
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
        .args(["watch", "-s"])
        .arg(
            "rm -rf target/docs/doc && mkdir -p target/docs/doc && CARGO_TARGET_DIR=target/docs CARGO_HOME=target/.cargo-docs-home RUSTDOCFLAGS='-D rustdoc::broken_intra_doc_links' cargo doc -p gig-log-api -p gig-log-common -p gig-log-dev-tools -p gig-log-frontend --no-deps --document-private-items --color always --locked && CARGO_TARGET_DIR=target/docs CARGO_HOME=target/.cargo-docs-home cargo run -p gig-log-dev-tools -- docs-index",
        )
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
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if the directory cannot be removed or
/// recreated.
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
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if index generation fails.
pub fn generate_index() -> Result<()> {
    doc_index::generate(DOCS_TARGET_DIR)
}

/// Verifies docs command prerequisites.
///
/// Checks whether `cargo-watch` and `miniserve` are available on the system
/// `PATH` before docs generation starts.
///
/// # Returns
///
/// An empty [`Result`] on success.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if either `cargo-watch` or `miniserve` is not
/// found on the system `PATH`.
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
