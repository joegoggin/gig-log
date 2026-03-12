use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::event::ModifyKind;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Web,
    Api,
    Common,
    DevTools,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IntentBatch {
    pub web: bool,
    pub api: bool,
    pub common: bool,
    pub dev_tools: bool,
}

impl IntentBatch {
    pub fn add(&mut self, intent: Intent) {
        match intent {
            Intent::Web => self.web = true,
            Intent::Api => self.api = true,
            Intent::Common => self.common = true,
            Intent::DevTools => self.dev_tools = true,
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.web && !self.api && !self.common && !self.dev_tools
    }

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

pub struct WatchStream {
    _watcher: RecommendedWatcher,
    rx: mpsc::UnboundedReceiver<Intent>,
}

impl WatchStream {
    pub async fn recv(&mut self) -> Option<Intent> {
        self.rx.recv().await
    }

    pub async fn collect_debounced_batch(
        &mut self,
        first: Intent,
        debounce: Duration,
    ) -> IntentBatch {
        collect_debounced_batch(first, &mut self.rx, debounce).await
    }
}

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
