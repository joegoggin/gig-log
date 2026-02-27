use std::collections::VecDeque;

const MAX_ENTRIES: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    Api,
    Web,
    Docs,
}

impl Service {
    pub fn label(&self) -> &'static str {
        match self {
            Service::Api => "API",
            Service::Web => "WEB",
            Service::Docs => "DOCS",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub service: Service,
    pub line: String,
}

pub struct LogStore {
    entries: VecDeque<LogEntry>,
}

impl LogStore {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::with_capacity(MAX_ENTRIES),
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        if self.entries.len() >= MAX_ENTRIES {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn filtered(&self, service: Option<Service>) -> Vec<&LogEntry> {
        match service {
            Some(s) => self.entries.iter().filter(|e| e.service == s).collect(),
            None => self.entries.iter().collect(),
        }
    }
}
