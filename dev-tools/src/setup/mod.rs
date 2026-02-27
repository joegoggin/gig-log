use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use dialoguer::{Input, Password};
use rand::distr::Alphanumeric;
use rand::{Rng, rng};
use tokio::process::Command;

pub struct SetupOptions {
    pub non_interactive: bool,
    pub skip_db: bool,
    pub skip_migrate: bool,
    pub skip_build: bool,
    pub dry_run: bool,
    pub release: bool,
}

struct SetupSummary {
    env_created: Vec<PathBuf>,
    env_skipped: Vec<PathBuf>,
    env_warnings: Vec<String>,
    db_started: bool,
    migrations_ran: bool,
    build_ran: bool,
}

impl SetupSummary {
    fn new() -> Self {
        Self {
            env_created: Vec::new(),
            env_skipped: Vec::new(),
            env_warnings: Vec::new(),
            db_started: false,
            migrations_ran: false,
            build_ran: false,
        }
    }
}

pub async fn run(options: SetupOptions) -> Result<()> {
    let mut summary = SetupSummary::new();

    println!("Running gig-log setup...");
    if options.dry_run {
        println!("Dry run enabled. Commands will not be executed.");
    }

    check_prerequisites(&options).await?;
    create_env_files(&options, &mut summary)?;

    if !options.skip_db {
        start_db_and_wait(&options).await?;
        summary.db_started = true;
    }

    if !options.skip_migrate {
        summary.migrations_ran = run_migrations(&options).await?;
    }

    if !options.skip_build {
        run_build(&options).await?;
        summary.build_ran = true;
    }

    print_summary(&summary, &options);
    Ok(())
}

async fn check_prerequisites(options: &SetupOptions) -> Result<()> {
    for tool in ["cargo", "docker"] {
        ensure_command_exists(tool, options).await?;
    }

    if !options.skip_migrate {
        ensure_command_exists("sqlx", options).await?;
    }

    if !options.skip_db {
        if options.dry_run {
            println!("[dry-run] Would verify Docker daemon is running");
        } else {
            let status = Command::new("docker")
                .arg("info")
                .status()
                .await
                .context("Failed to run docker info")?;
            if !status.success() {
                anyhow::bail!("Docker daemon is not running. Start Docker and run setup again.");
            }
        }
    }

    Ok(())
}

async fn ensure_command_exists(command: &str, options: &SetupOptions) -> Result<()> {
    if options.dry_run {
        println!("[dry-run] Would verify command exists: {command}");
        return Ok(());
    }

    let status = Command::new("which")
        .arg(command)
        .status()
        .await
        .with_context(|| format!("Failed to check command: {command}"))?;
    if !status.success() {
        anyhow::bail!("Required command not found: {command}");
    }
    Ok(())
}

fn create_env_files(options: &SetupOptions, summary: &mut SetupSummary) -> Result<()> {
    let templates = discover_env_templates()?;
    if templates.is_empty() {
        println!("No .env.example templates found.");
        return Ok(());
    }

    for template in templates {
        let target = template
            .to_string_lossy()
            .strip_suffix(".example")
            .map(PathBuf::from)
            .context("Invalid env template path")?;

        if target.exists() {
            summary.env_skipped.push(target);
            continue;
        }

        let template_text = fs::read_to_string(&template)
            .with_context(|| format!("Failed to read template {}", template.display()))?;
        let rendered = render_env_template(&template_text, &template, options, summary)?;

        if options.dry_run {
            println!(
                "[dry-run] Would create env file from template: {} -> {}",
                template.display(),
                target.display()
            );
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create parent directory {}", parent.display())
                })?;
            }
            fs::write(&target, rendered)
                .with_context(|| format!("Failed to write env file {}", target.display()))?;
        }

        summary.env_created.push(target);
    }

    Ok(())
}

fn discover_env_templates() -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_env_examples(Path::new("."), &mut out)?;
    out.sort();
    Ok(out)
}

fn collect_env_examples(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("Failed to read directory {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if matches!(name, ".git" | "target" | "node_modules") {
                continue;
            }
            collect_env_examples(&path, out)?;
            continue;
        }

        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|name| name.ends_with(".env.example"))
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
    Ok(())
}

fn render_env_template(
    template_text: &str,
    template_path: &Path,
    options: &SetupOptions,
    summary: &mut SetupSummary,
) -> Result<String> {
    let mut values: BTreeMap<String, String> = BTreeMap::new();

    for line in template_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };

        let key = key.trim().to_string();
        let raw = value.trim();
        if needs_user_value(&key, raw) {
            let suggested = normalize_value(raw);
            let resolved = resolve_sensitive_value(&key, &suggested, template_path, options)?;
            values.insert(key, resolved);
        }
    }

    let mut rendered = String::with_capacity(template_text.len() + 64);
    for line in template_text.lines() {
        let trimmed = line.trim();
        if let Some((key, _)) = trimmed.split_once('=') {
            let clean_key = key.trim();
            if let Some(new_value) = values.get(clean_key) {
                rendered.push_str(clean_key);
                rendered.push('=');
                rendered.push_str(new_value);
                rendered.push('\n');
                continue;
            }
        }
        rendered.push_str(line);
        rendered.push('\n');
    }

    if rendered.contains("your_") || rendered.contains("example.com") {
        summary.env_warnings.push(format!(
            "Template {} still includes placeholder-like values; review created file.",
            template_path.display()
        ));
    }

    Ok(rendered)
}

fn resolve_sensitive_value(
    key: &str,
    suggested: &str,
    template_path: &Path,
    options: &SetupOptions,
) -> Result<String> {
    if options.non_interactive {
        if key == "JWT_SECRET" {
            return Ok(generate_jwt_secret());
        }
        anyhow::bail!(
            "Missing required value for {key} in {} while in --non-interactive mode",
            template_path.display()
        );
    }

    let prompt = format!("Enter value for {key} ({})", template_path.display());
    let input = if is_secret_key(key) {
        Password::new()
            .with_prompt(prompt)
            .allow_empty_password(true)
            .interact()
            .context("Failed to read input")?
    } else {
        Input::<String>::new()
            .with_prompt(prompt)
            .with_initial_text(suggested)
            .allow_empty(true)
            .interact_text()
            .context("Failed to read input")?
    };

    if key == "JWT_SECRET" && input.trim().is_empty() {
        return Ok(generate_jwt_secret());
    }

    if input.trim().is_empty() {
        anyhow::bail!("Value for {key} cannot be empty");
    }

    Ok(format_env_value(&input))
}

fn needs_user_value(key: &str, raw_value: &str) -> bool {
    let value = normalize_value(raw_value);
    key == "JWT_SECRET"
        || value.contains("your_")
        || value.contains("example.com")
        || value.contains("yourdomain")
        || value.contains("re_your")
}

fn normalize_value(value: &str) -> String {
    let mut out = value.trim().to_string();
    if out.starts_with('"') && out.ends_with('"') && out.len() >= 2 {
        out = out[1..out.len() - 1].to_string();
    }
    out
}

fn is_secret_key(key: &str) -> bool {
    key.contains("SECRET") || key.contains("PASSWORD") || key.contains("API_KEY")
}

fn format_env_value(value: &str) -> String {
    if value.contains(' ') {
        format!("\"{value}\"")
    } else {
        value.to_string()
    }
}

fn generate_jwt_secret() -> String {
    let rng = rng();
    rng.sample_iter(&Alphanumeric)
        .map(char::from)
        .take(64)
        .collect()
}

async fn start_db_and_wait(options: &SetupOptions) -> Result<()> {
    run_command(
        options,
        "docker",
        &["compose", "up", "-d", "postgres"],
        "starting postgres",
    )
    .await?;

    if options.dry_run {
        println!("[dry-run] Would wait for postgres readiness");
        return Ok(());
    }

    const ATTEMPTS: usize = 20;
    for _ in 0..ATTEMPTS {
        let status = Command::new("docker")
            .args([
                "compose",
                "exec",
                "-T",
                "postgres",
                "sh",
                "-lc",
                "pg_isready -U \"$POSTGRES_USER\" -d \"$POSTGRES_DB\"",
            ])
            .status()
            .await;

        if matches!(status, Ok(s) if s.success()) {
            println!("Postgres is ready.");
            return Ok(());
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    anyhow::bail!("Postgres did not become ready in time")
}

async fn run_migrations(options: &SetupOptions) -> Result<bool> {
    if !Path::new("migrations").exists() {
        println!("No migrations directory found. Skipping sqlx migrate run.");
        return Ok(false);
    }

    run_command(options, "sqlx", &["migrate", "run"], "running migrations").await?;
    Ok(true)
}

async fn run_build(options: &SetupOptions) -> Result<()> {
    if options.release {
        run_command(
            options,
            "cargo",
            &["build", "--workspace", "--release"],
            "building workspace release",
        )
        .await
    } else {
        run_command(
            options,
            "cargo",
            &["build", "--workspace"],
            "building workspace",
        )
        .await
    }
}

async fn run_command(options: &SetupOptions, cmd: &str, args: &[&str], description: &str) -> Result<()> {
    if options.dry_run {
        println!("[dry-run] Would run ({description}): {cmd} {}", args.join(" "));
        return Ok(());
    }

    let status = Command::new(cmd)
        .args(args)
        .status()
        .await
        .with_context(|| format!("Failed {description}"))?;
    if !status.success() {
        anyhow::bail!("Command failed while {description}: {cmd} {}", args.join(" "));
    }
    Ok(())
}

fn print_summary(summary: &SetupSummary, options: &SetupOptions) {
    println!();
    println!("Setup complete.");
    println!("- Env files created: {}", summary.env_created.len());
    for path in &summary.env_created {
        println!("  - {}", path.display());
    }

    println!("- Env files skipped (already exist): {}", summary.env_skipped.len());
    for path in &summary.env_skipped {
        println!("  - {}", path.display());
    }

    for warning in &summary.env_warnings {
        println!("- Warning: {warning}");
    }

    if options.skip_db {
        println!("- Database startup: skipped");
    } else {
        println!("- Database startup: {}", if summary.db_started { "done" } else { "not run" });
    }

    if options.skip_migrate {
        println!("- Migrations: skipped");
    } else {
        println!("- Migrations: {}", if summary.migrations_ran { "done" } else { "not run" });
    }

    if options.skip_build {
        println!("- Build: skipped");
    } else {
        println!("- Build: {}", if summary.build_ran { "done" } else { "not run" });
    }

    println!("- Existing env files were left untouched (non-destructive mode).");
    println!("Next step: run `just dev`");
}
