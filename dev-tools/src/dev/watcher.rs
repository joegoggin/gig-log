//! Filesystem watch classification and debounce batching.
//!
//! This module maps raw notify events into orchestrator intents so rebuild and
//! restart work can be executed in minimal batches.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::event::ModifyKind;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

/// Identifies which orchestrator workflow should run for a path change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Rebuild docs after web-related changes.
    Web,
    /// Restart the API process and run docs generation.
    Api,
    /// Rebuild common, then restart API and regenerate docs.
    Common,
    /// Rebuild dev-tools artifacts and regenerate docs.
    DevTools,
}

/// Represents a merged set of change intents within one debounce window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntentBatch {
    /// Indicates whether web intent is present.
    pub web: bool,
    /// Indicates whether api intent is present.
    pub api: bool,
    /// Indicates whether common intent is present.
    pub common: bool,
    /// Indicates whether dev-tools intent is present.
    pub dev_tools: bool,
}

impl IntentBatch {
    /// Adds a single intent to the batch.
    ///
    /// # Arguments
    ///
    /// * `intent` — Intent to merge into this batch.
    pub fn add(&mut self, intent: Intent) {
        match intent {
            Intent::Web => self.web = true,
            Intent::Api => self.api = true,
            Intent::Common => self.common = true,
            Intent::DevTools => self.dev_tools = true,
        }
    }

    /// Returns whether no intents are currently set.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether all intent flags are `false`.
    pub fn is_empty(&self) -> bool {
        !self.web && !self.api && !self.common && !self.dev_tools
    }

    /// Collapses overlapping intents into an execution-safe set.
    ///
    /// When `common` is set, `api` and `web` become redundant because common
    /// changes already force downstream API and docs work.
    ///
    /// # Returns
    ///
    /// An [`IntentBatch`] normalized for command execution.
    pub fn merged_for_execution(self) -> Self {
        if self.common {
            Self {
                web: false,
                api: false,
                common: true,
                dev_tools: self.dev_tools,
            }
        } else {
            self
        }
    }
}

/// Classifies a changed path into an orchestrator intent.
///
/// # Arguments
///
/// * `path` — Workspace-relative file path reported by the watcher.
///
/// # Returns
///
/// An optional [`Intent`] when the path maps to a watched workspace area.
pub fn classify_path(path: &Path) -> Option<Intent> {
    if is_ignored_generated_path(path) {
        return None;
    }

    let first = path.components().next()?.as_os_str().to_string_lossy();
    match first.as_ref() {
        "web" => Some(Intent::Web),
        "api" => Some(Intent::Api),
        "common" => Some(Intent::Common),
        "dev-tools" => Some(Intent::DevTools),
        _ => None,
    }
}

/// Detects generated output paths that should not trigger rebuilds.
///
/// # Arguments
///
/// * `path` — Workspace-relative path to evaluate.
///
/// # Returns
///
/// A boolean indicating whether the path should be ignored.
fn is_ignored_generated_path(path: &Path) -> bool {
    let mut components = path.components();
    let first = match components.next() {
        Some(component) => component.as_os_str().to_string_lossy(),
        None => return false,
    };

    let second = components
        .next()
        .map(|component| component.as_os_str().to_string_lossy());

    if first == "web" {
        if let Some(second) = second {
            if second == "dist" || second == ".trunk" {
                return true;
            }
        }
    }

    false
}

/// Filters notify events to those that imply meaningful source changes.
///
/// # Arguments
///
/// * `kind` — Notify event kind emitted by the filesystem watcher.
///
/// # Returns
///
/// A boolean indicating whether the event should be processed.
pub fn is_relevant_event(kind: &EventKind) -> bool {
    match kind {
        EventKind::Create(_) | EventKind::Remove(_) => true,
        EventKind::Modify(ModifyKind::Data(_))
        | EventKind::Modify(ModifyKind::Name(_))
        | EventKind::Modify(ModifyKind::Metadata(_))
        | EventKind::Modify(ModifyKind::Any)
        | EventKind::Modify(ModifyKind::Other) => true,
        _ => false,
    }
}

/// Provides asynchronous access to classified watch intents.
pub struct WatchStream {
    /// Holds the underlying watcher to keep it alive for the stream lifetime.
    _watcher: RecommendedWatcher,
    /// Receives classified intents from the watcher callback.
    rx: mpsc::UnboundedReceiver<Intent>,
}

impl WatchStream {
    /// Receives the next classified intent from the watcher.
    ///
    /// # Returns
    ///
    /// An optional [`Intent`] when the watcher is still active.
    pub async fn recv(&mut self) -> Option<Intent> {
        self.rx.recv().await
    }

    /// Collects a debounced batch beginning with the first intent.
    ///
    /// # Arguments
    ///
    /// * `first` — First intent already received.
    /// * `debounce` — Debounce interval used to merge nearby events.
    ///
    /// # Returns
    ///
    /// An [`IntentBatch`] containing merged intents for one execution window.
    pub async fn collect_debounced_batch(
        &mut self,
        first: Intent,
        debounce: Duration,
    ) -> IntentBatch {
        collect_debounced_batch(first, &mut self.rx, debounce).await
    }
}

/// Starts recursive watchers for the provided workspace paths.
///
/// # Arguments
///
/// * `paths` — Root directories to monitor recursively.
///
/// # Returns
///
/// A [`WatchStream`] that yields classified intents.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] if watcher creation fails or a path cannot be watched.
pub fn start(paths: &[&str]) -> Result<WatchStream> {
    let (tx, rx) = mpsc::unbounded_channel::<Intent>();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        let event = match res {
            Ok(event) => event,
            Err(_) => return,
        };

        if !is_relevant_event(&event.kind) {
            return;
        }

        for path in event.paths {
            let relative = normalize_to_workspace_relative(&path);
            if let Some(intent) = classify_path(&relative) {
                let _ = tx.send(intent);
            }
        }
    })
    .context("Failed to create file watcher")?;

    for &path in paths {
        watcher
            .watch(Path::new(path), RecursiveMode::Recursive)
            .with_context(|| format!("Failed to watch path: {path}"))?;
    }

    Ok(WatchStream {
        _watcher: watcher,
        rx,
    })
}

/// Collects intents until the debounce window expires.
///
/// # Arguments
///
/// * `first` — First intent already received before debounce collection begins.
/// * `rx` — Intent receiver to drain during the debounce window.
/// * `debounce` — Duration to wait for additional intents.
///
/// # Returns
///
/// An [`IntentBatch`] with all intents merged during the window.
pub async fn collect_debounced_batch(
    first: Intent,
    rx: &mut mpsc::UnboundedReceiver<Intent>,
    debounce: Duration,
) -> IntentBatch {
    let mut batch = IntentBatch::default();
    batch.add(first);

    let delay = tokio::time::sleep(debounce);
    tokio::pin!(delay);

    loop {
        tokio::select! {
            maybe_intent = rx.recv() => {
                match maybe_intent {
                    Some(intent) => {
                        batch.add(intent);
                        delay.as_mut().reset(tokio::time::Instant::now() + debounce);
                    }
                    None => break,
                }
            }
            _ = &mut delay => {
                break;
            }
        }
    }

    batch
}

/// Converts an absolute path to a workspace-relative path when possible.
///
/// # Arguments
///
/// * `path` — Path reported by notify.
///
/// # Returns
///
/// A [`PathBuf`] relative to the current workspace, or the original path.
fn normalize_to_workspace_relative(path: &Path) -> PathBuf {
    if path.is_relative() {
        return path.to_path_buf();
    }

    let cwd = std::env::current_dir().ok();
    if let Some(cwd) = cwd {
        if let Ok(relative) = path.strip_prefix(cwd) {
            return relative.to_path_buf();
        }
    }

    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::{Intent, IntentBatch, classify_path};

    #[test]
    fn classify_paths() {
        assert_eq!(classify_path("web/src/main.rs".as_ref()), Some(Intent::Web));
        assert_eq!(classify_path("api/src/main.rs".as_ref()), Some(Intent::Api));
        assert_eq!(
            classify_path("common/src/lib.rs".as_ref()),
            Some(Intent::Common)
        );
        assert_eq!(
            classify_path("dev-tools/src/main.rs".as_ref()),
            Some(Intent::DevTools)
        );
        assert_eq!(classify_path("web/dist/index.html".as_ref()), None);
        assert_eq!(classify_path("web/.trunk/state".as_ref()), None);
        assert_eq!(classify_path("README.md".as_ref()), None);
    }

    #[test]
    fn common_suppresses_api_and_web() {
        let mut batch = IntentBatch::default();
        batch.add(Intent::Web);
        batch.add(Intent::Api);
        batch.add(Intent::Common);

        let merged = batch.merged_for_execution();
        assert!(merged.common);
        assert!(!merged.web);
        assert!(!merged.api);
    }
}
