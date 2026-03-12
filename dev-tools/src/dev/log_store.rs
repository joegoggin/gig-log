#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    Api,
    Web,
    Common,
    DevTools,
    Docs,
    System,
}

impl Service {
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

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub service: Service,
    pub line: String,
}

pub struct LogStore {
    entries: Vec<LogEntry>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn clear_filtered(&mut self, service: Option<Service>) {
        match service {
            Some(service) => self.entries.retain(|entry| entry.service != service),
            None => self.clear(),
        }
    }

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
