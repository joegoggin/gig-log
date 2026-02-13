//! Docker Compose auto-start checks for local development startup.
//!
//! This module ensures the project's Docker Compose stack is started before
//! API boot when the runtime environment is development.

use std::{collections::HashSet, env, path::PathBuf, process::Command};

use anyhow::{Context, anyhow};

use crate::core::{app::AppResult, env::Env, logger::Logger};

/// Ensures Docker Compose services are up when booting in development mode.
///
/// This check is intentionally skipped outside development so production and
/// test environments are not coupled to local Docker startup behavior.
///
/// # Arguments
///
/// - `env` - Runtime environment used to determine whether checks should run.
///
/// # Errors
///
/// Returns an error when Docker Compose cannot be started in development mode.
pub async fn ensure_docker_compose_ready_for_dev(env: &Env) -> AppResult<()> {
    if !env.is_development() {
        return Ok(());
    }

    Logger::log_message("Ensuring Docker Compose services are running");
    ensure_compose_stack_running().context("development Docker Compose auto-start failed")
}

fn ensure_compose_stack_running() -> AppResult<()> {
    let compose_file = resolve_compose_file_path()?;
    let compose_file_arg = compose_file.to_string_lossy().to_string();

    if compose_services_already_running(&compose_file_arg) {
        Logger::log_success("Docker Compose services are already running");
        return Ok(());
    }

    Logger::log_message("Running Docker Compose command");

    let status = Command::new("docker")
        .args(["compose", "-f", &compose_file_arg, "up", "-d"])
        .status()
        .context("failed to execute `docker compose` command")?;

    if status.success() {
        Logger::log_success("Docker Compose services are running");
        return Ok(());
    }

    Err(anyhow!(
        "Failed to start Docker Compose stack using `{}`. Ensure Docker daemon is running and try again. Exit status: {}",
        compose_file_arg,
        status
    ))
}

fn compose_services_already_running(compose_file_arg: &str) -> bool {
    let defined_output = Command::new("docker")
        .args(["compose", "-f", compose_file_arg, "config", "--services"])
        .output();

    let running_output = Command::new("docker")
        .args([
            "compose",
            "-f",
            compose_file_arg,
            "ps",
            "--services",
            "--filter",
            "status=running",
        ])
        .output();

    let (Ok(defined_output), Ok(running_output)) = (defined_output, running_output) else {
        return false;
    };

    if !defined_output.status.success() || !running_output.status.success() {
        return false;
    }

    let defined_services = parse_services(&defined_output.stdout);
    let running_services = parse_services(&running_output.stdout);

    !defined_services.is_empty() && defined_services.is_subset(&running_services)
}

fn parse_services(bytes: &[u8]) -> HashSet<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn resolve_compose_file_path() -> AppResult<PathBuf> {
    let candidates = [
        "docker-compose.yaml",
        "docker-compose.yml",
        "compose.yaml",
        "compose.yml",
    ];

    let cwd = env::current_dir().context("failed to resolve current directory")?;
    let search_dirs = [cwd.clone(), cwd.join("..")];

    for directory in search_dirs {
        for candidate in candidates {
            let path = directory.join(candidate);
            if path.exists() {
                return Ok(path);
            }
        }
    }

    Err(anyhow!(
        "Could not find a Docker Compose file. Expected one of: docker-compose.yaml, docker-compose.yml, compose.yaml, compose.yml in current or parent directory."
    ))
}

#[cfg(test)]
mod tests {
    //! Unit tests for environment gating helpers used by Docker auto-start.

    use crate::core::env::Env;

    #[test]
    fn recognizes_development_environment_values() {
        assert!(Env::is_development_env("development"));
        assert!(Env::is_development_env("dev"));
        assert!(Env::is_development_env("DeVeLoPmEnT"));
        assert!(Env::is_development_env("  dev  "));
    }

    #[test]
    fn rejects_non_development_environment_values() {
        assert!(!Env::is_development_env("production"));
        assert!(!Env::is_development_env("staging"));
        assert!(!Env::is_development_env(""));
    }
}
