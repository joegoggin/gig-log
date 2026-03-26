use std::path::PathBuf;

use anyhow::Context;

fn workspace_root() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path
}

pub fn workspace_env_path() -> PathBuf {
    workspace_root().join(".env")
}

pub fn load_workspace_env() -> anyhow::Result<()> {
    let env_path = workspace_env_path();
    if env_path.exists() {
        dotenvy::from_path(&env_path).with_context(|| {
            format!("failed to load environment file at {}", env_path.display())
        })?;
    }

    Ok(())
}

pub fn required_var(name: &str) -> anyhow::Result<String> {
    match std::env::var(name) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => {
            let env_path = workspace_env_path();
            anyhow::bail!(
                "Missing required environment variable `{name}`.\n\
Tried process environment and {}.\n\
Add `{name}=...` to that .env file or export it before running the command.",
                env_path.display()
            )
        }
    }
}
