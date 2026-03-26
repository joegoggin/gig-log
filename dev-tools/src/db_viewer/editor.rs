use std::fs;
use std::io::Write;
use std::process::Command;

use anyhow::Context;
use tempfile::NamedTempFile;

pub fn open_external_editor(initial_content: &str, suffix: &str) -> anyhow::Result<String> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut temp_file = NamedTempFile::with_suffix(suffix)
        .with_context(|| format!("failed to create temporary {suffix} file"))?;

    temp_file
        .write_all(initial_content.as_bytes())
        .context("failed to write initial editor content")?;
    temp_file.flush()?;

    let temp_path = temp_file.path().to_owned();
    let status = Command::new(&editor)
        .arg(&temp_path)
        .status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;

    if !status.success() {
        anyhow::bail!("editor exited with non-zero status: {status}");
    }

    fs::read_to_string(&temp_path).context("failed to read edited content")
}
