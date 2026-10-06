//! User-facing presentation: events, reporters, and shared formatters.

mod format;
mod mode;
mod stdout;
pub mod tui;

pub use format::{format_ignore_yaml, format_resolution_line, format_section_lines};
pub use mode::{UiMode, select_ui_mode};
pub use stdout::StdoutReporter;

use std::sync::Arc;

/// Kind of live activity line (maps to tagged stdout prefixes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Yarn,
    Hit,
    Info,
    Npm,
}

/// Coarse phase for the live dashboard chrome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Install,
    Dedupe,
    Audit,
    Fix,
    Hygiene,
    Report,
    Done,
}

/// Structured UX events emitted by business logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    Activity {
        kind: ActivityKind,
        message: String,
    },
    Fix {
        message: String,
    },
    Phase(Phase),
    Kpis {
        dependencies: usize,
        dev_dependencies: usize,
        locks: usize,
        resolutions: usize,
        deprecations: usize,
        unresolved_issues: usize,
    },
    Section {
        title: String,
        lines: Vec<String>,
    },
    /// Paste-ready ignore YAML (no section title); same as bare stdout today.
    IgnoreYaml {
        yaml: String,
    },
    /// Structured outside-range pins (package key + version, no caret). Stdout ignores this.
    SuggestedResolutions {
        entries: Vec<(String, String)>,
    },
    /// Structured suggested-ignore advisory IDs. Stdout ignores this.
    SuggestedIgnoreIds {
        ids: Vec<String>,
    },
    ReportComplete,
    Error {
        message: String,
    },
}

/// Sink for [`UiEvent`]s. Implementations must be cheap to clone via [`Arc`].
pub trait Reporter: Send + Sync {
    fn emit(&self, event: UiEvent);
}

pub type SharedReporter = Arc<dyn Reporter>;

pub fn stdout_reporter() -> SharedReporter {
    Arc::new(StdoutReporter)
}

/// Test double that records events.
#[derive(Default)]
pub struct CollectingReporter {
    events: std::sync::Mutex<Vec<UiEvent>>,
}

impl CollectingReporter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<UiEvent> {
        self.events.lock().expect("collector poisoned").clone()
    }
}

impl Reporter for CollectingReporter {
    fn emit(&self, event: UiEvent) {
        self.events
            .lock()
            .expect("collector poisoned")
            .push(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collecting_reporter_records_events() {
        let reporter = CollectingReporter::new();
        reporter.emit(UiEvent::Activity {
            kind: ActivityKind::Info,
            message: "hello".into(),
        });
        reporter.emit(UiEvent::Fix {
            message: "reset foo".into(),
        });
        let events = reporter.events();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            &events[0],
            UiEvent::Activity {
                kind: ActivityKind::Info,
                message
            } if message == "hello"
        ));
    }
}
