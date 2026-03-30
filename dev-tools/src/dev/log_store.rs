//! In-memory log storage and service labeling for the dev orchestrator.
//!
//! This module defines service channels, log entry records, and filtering
//! operations used by the terminal UI.

/// Represents a log-producing service within the orchestrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    /// Indicates the API service process.
    Api,
    /// Indicates the web service process.
    Web,
    /// Indicates the shared common crate build step.
    Common,
    /// Indicates the dev-tools crate build step.
    DevTools,
    /// Indicates docs generation and docs-serving workflows.
    Docs,
    /// Indicates orchestrator/system lifecycle messages.
    System,
}

impl Service {
    /// Returns the uppercase display label for the service.
    ///
    /// # Returns
    ///
    /// A static string label used by log headers and filters.
    pub fn label(&self) -> &'static str {
        match self {
            Service::Api => "API",
            Service::Web => "WEB",
            Service::Common => "COMMON",
            Service::DevTools => "DEV-TOOLS",
            Service::Docs => "DOCS",
            Service::System => "SYSTEM",
        }
    }
}

/// Stores a single emitted log line with its source service.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Identifies which service produced the line.
    pub service: Service,
    /// Contains the rendered log content.
    pub line: String,
}

/// Holds log entries and provides service-level filtering operations.
pub struct LogStore {
    /// Accumulates logs in insertion order.
    entries: Vec<LogEntry>,
}

impl LogStore {
    /// Creates an empty log store.
    ///
    /// # Returns
    ///
    /// An empty [`LogStore`] on success.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Appends a log entry to the store.
    ///
    /// # Arguments
    ///
    /// * `entry` — Log entry to append.
    pub fn push(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    /// Removes all log entries from the store.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Removes entries for a specific service, or all entries.
    ///
    /// # Arguments
    ///
    /// * `service` — Service filter to clear, or `None` to clear all entries.
    pub fn clear_filtered(&mut self, service: Option<Service>) {
        match service {
            Some(service) => self.entries.retain(|entry| entry.service != service),
            None => self.clear(),
        }
    }

    /// Returns entries filtered by service when requested.
    ///
    /// # Arguments
    ///
    /// * `service` — Service filter to apply, or `None` for all entries.
    ///
    /// # Returns
    ///
    /// A vector of references to matching [`LogEntry`] values.
    pub fn filtered(&self, service: Option<Service>) -> Vec<&LogEntry> {
        match service {
            Some(s) => self.entries.iter().filter(|e| e.service == s).collect(),
            None => self.entries.iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LogEntry, LogStore, Service};

    #[test]
    fn clear_filtered_removes_only_selected_service() {
        let mut store = LogStore::new();
        store.push(LogEntry {
            service: Service::Api,
            line: "api 1".to_string(),
        });
        store.push(LogEntry {
            service: Service::Web,
            line: "web 1".to_string(),
        });
        store.push(LogEntry {
            service: Service::Api,
            line: "api 2".to_string(),
        });
        store.push(LogEntry {
            service: Service::Docs,
            line: "docs 1".to_string(),
        });

        store.clear_filtered(Some(Service::Api));

        let remaining = store.filtered(None);
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].service, Service::Web);
        assert_eq!(remaining[0].line, "web 1");
        assert_eq!(remaining[1].service, Service::Docs);
        assert_eq!(remaining[1].line, "docs 1");
    }

    #[test]
    fn clear_filtered_none_removes_all_entries() {
        let mut store = LogStore::new();
        store.push(LogEntry {
            service: Service::Api,
            line: "api".to_string(),
        });
        store.push(LogEntry {
            service: Service::Web,
            line: "web".to_string(),
        });

        store.clear_filtered(None);

        assert!(store.filtered(None).is_empty());
    }

    #[test]
    fn clear_filtered_is_no_op_when_service_not_present() {
        let mut store = LogStore::new();
        store.push(LogEntry {
            service: Service::Web,
            line: "web".to_string(),
        });
        store.push(LogEntry {
            service: Service::Docs,
            line: "docs".to_string(),
        });

        store.clear_filtered(Some(Service::Api));

        let remaining = store.filtered(None);
        assert_eq!(remaining.len(), 2);
        assert_eq!(remaining[0].service, Service::Web);
        assert_eq!(remaining[0].line, "web");
        assert_eq!(remaining[1].service, Service::Docs);
        assert_eq!(remaining[1].line, "docs");
    }
}
