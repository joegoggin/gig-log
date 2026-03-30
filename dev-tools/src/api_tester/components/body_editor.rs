//! External editor integration for request body editing.

use std::fs;
use std::io::Write;
use std::process::Command;

use anyhow::Context;
use tempfile::NamedTempFile;

/// Opens the configured `$EDITOR` with request body content.
///
/// Creates a temporary `.json` file, writes optional initial content, launches
/// the editor process, then reads the edited content back.
///
/// # Arguments
///
/// * `initial_content` — Optional initial body text to seed the editor file.
///
/// # Returns
///
/// An [`Option`] containing edited body text, or `None` when the final content
/// is empty.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if temporary file operations, editor launch,
/// or content reading fails.
pub fn open_external_editor(initial_content: Option<&str>) -> anyhow::Result<Option<String>> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    let mut temp_file =
        NamedTempFile::with_suffix(".json").context("failed to create temporary file")?;

    if let Some(content) = initial_content {
        temp_file
            .write_all(content.as_bytes())
            .context("failed to write initial content to temp file")?;
        temp_file.flush()?;
    }

    let temp_path = temp_file.path().to_owned();

    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;

    if !status.success() {
        anyhow::bail!("editor exited with non-zero status: {status}");
    }

    let content = fs::read_to_string(&temp_path).context("failed to read edited content")?;

    if content.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(content))
    }
}
