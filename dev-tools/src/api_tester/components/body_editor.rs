use std::fs;
use std::io::Write;
use std::process::Command;

use anyhow::Context;
use tempfile::NamedTempFile;

/// Opens the user's $EDITOR with the given content in a temporary file.
/// Returns the edited content, or None if the user quit without saving
/// or the content is empty.
pub fn open_external_editor(initial_content: Option<&str>) -> anyhow::Result<Option<String>> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    // Create a temp file with .json extension for syntax highlighting
    let mut temp_file =
        NamedTempFile::with_suffix(".json").context("failed to create temporary file")?;

    if let Some(content) = initial_content {
        temp_file
            .write_all(content.as_bytes())
            .context("failed to write initial content to temp file")?;
        temp_file.flush()?;
    }

    let temp_path = temp_file.path().to_owned();

    // Launch the editor as a child process
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;

    if !status.success() {
        anyhow::bail!("editor exited with non-zero status: {status}");
    }

    // Read back the content
    let content = fs::read_to_string(&temp_path).context("failed to read edited content")?;

    let trimmed = content.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed.to_string()))
    }
}
