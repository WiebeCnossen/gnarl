use std::time::Duration;

use crate::cmd::Verb;
use crate::ui::{ActivityKind, Phase, UiEvent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneId {
    Activity = 0,
    Fixed = 1,
    State = 2,
    Next = 3,
}

impl PaneId {
    pub const ALL: [PaneId; 4] = [
        PaneId::Activity,
        PaneId::Fixed,
        PaneId::State,
        PaneId::Next,
    ];

    pub fn title(self, state: &UiState) -> String {
        match self {
            PaneId::Activity => " ACTIVITY ".into(),
            PaneId::Fixed => format!(" FIXED ({}) ", state.fixes.len()),
            PaneId::State => " INSIGHTS ".into(),
            PaneId::Next => format!(" TASKS ({}) ", state.suggested_ignores.len()),
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub verb: Verb,
    pub phase: Phase,
    pub elapsed: Duration,
    /// Full activity history (no truncation).
    pub activity: Vec<String>,
    pub fixes: Vec<String>,
    pub report_ready: bool,
    pub done: bool,
    pub worker_finished: bool,
    pub kpis: Option<KpiSnapshot>,
    pub ignore_overview: Vec<String>,
    pub deprecations: Vec<String>,
    pub fixes_section: Vec<String>,
    pub suggested_resolutions: Vec<String>,
    pub unresolved: Vec<String>,
    pub suggested_ignores: Vec<String>,
    pub ignore_yaml: Option<String>,
    pub error: Option<String>,
    pub focus: PaneId,
    /// Focused pane fills the grid area (Enter); Esc restores the 2×2 layout.
    pub maximized: bool,
    /// Lines scrolled up from the bottom of each pane (0 = follow latest).
    pub scroll_from_bottom: [usize; 4],
}

#[derive(Debug, Clone)]
pub struct KpiSnapshot {
    pub dependencies: usize,
    pub dev_dependencies: usize,
    pub locks: usize,
    pub resolutions: usize,
    pub deprecations: usize,
    pub unresolved_issues: usize,
}

impl UiState {
    pub fn new(verb: Verb) -> Self {
        Self {
            verb,
            phase: match verb {
                Verb::Check => Phase::Report,
                _ => Phase::Install,
            },
            elapsed: Duration::ZERO,
            activity: Vec::new(),
            fixes: Vec::new(),
            report_ready: false,
            done: false,
            worker_finished: false,
            kpis: None,
            ignore_overview: Vec::new(),
            deprecations: Vec::new(),
            fixes_section: Vec::new(),
            suggested_resolutions: Vec::new(),
            unresolved: Vec::new(),
            suggested_ignores: Vec::new(),
            ignore_yaml: None,
            error: None,
            focus: PaneId::Activity,
            maximized: false,
            scroll_from_bottom: [0; 4],
        }
    }

    pub fn apply(&mut self, event: UiEvent) {
        match event {
            UiEvent::Activity { kind, message } => {
                let prefix = match kind {
                    ActivityKind::Yarn => "[YARN]",
                    ActivityKind::Hit => "[HIT#]",
                    ActivityKind::Info => "[INFO]",
                    ActivityKind::Npm => "[NPM?]",
                };
                self.activity.push(format!("{prefix} {message}"));
            }
            UiEvent::Fix { message } => {
                self.fixes.push(message.clone());
                self.activity.push(format!("[FIX!] {message}"));
            }
            UiEvent::Phase(phase) => {
                self.phase = phase;
                if phase == Phase::Done {
                    self.done = true;
                }
            }
            UiEvent::Kpis {
                dependencies,
                dev_dependencies,
                locks,
                resolutions,
                deprecations,
                unresolved_issues,
            } => {
                self.report_ready = true;
                self.kpis = Some(KpiSnapshot {
                    dependencies,
                    dev_dependencies,
                    locks,
                    resolutions,
                    deprecations,
                    unresolved_issues,
                });
            }
            UiEvent::Section { title, lines } => {
                self.report_ready = true;
                match title.as_str() {
                    "npmAuditIgnoreAdvisories" => self.ignore_overview = lines,
                    "deprecations" => self.deprecations = lines,
                    "fixes" => self.fixes_section = lines,
                    "suggested resolutions" => self.suggested_resolutions = lines,
                    "unresolved issues" => self.unresolved = lines,
                    "suggested ignores" => self.suggested_ignores = lines,
                    _ => self.activity.push(format!("[INFO] {title}")),
                }
            }
            UiEvent::IgnoreYaml { yaml } => {
                self.report_ready = true;
                self.ignore_yaml = Some(yaml);
            }
            UiEvent::ReportComplete => {
                self.report_ready = true;
                self.done = true;
                self.phase = Phase::Done;
            }
            UiEvent::Error { message } => {
                self.error = Some(message);
            }
        }
    }

    pub fn cycle_focus(&mut self) {
        let idx = self.focus.index();
        self.focus = PaneId::ALL[(idx + 1) % PaneId::ALL.len()];
    }

    pub fn cycle_focus_backward(&mut self) {
        let idx = self.focus.index();
        let len = PaneId::ALL.len();
        self.focus = PaneId::ALL[(idx + len - 1) % len];
    }

    pub fn scroll_focused(&mut self, delta: isize, viewport_height: usize) {
        let pane = self.focus;
        let len = self.pane_lines(pane).len();
        let max_scroll = len.saturating_sub(viewport_height.max(1));
        let idx = pane.index();
        let current = self.scroll_from_bottom[idx] as isize;
        let next = (current + delta).clamp(0, max_scroll as isize);
        self.scroll_from_bottom[idx] = next as usize;
    }

    pub fn pane_lines(&self, pane: PaneId) -> Vec<String> {
        match pane {
            PaneId::Activity => self.activity.clone(),
            PaneId::Fixed => self.fixes.clone(),
            PaneId::State => self.state_lines(),
            PaneId::Next => self.next_lines(),
        }
    }

    pub fn state_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.report_ready {
            lines.push("— waiting for report —".into());
            return lines;
        }
        if let Some(k) = &self.kpis {
            lines.push(format!(
                "KPIs  deps {}  dev {}  locks {}  resolutions {}  deprecations {}  unresolved {}",
                k.dependencies,
                k.dev_dependencies,
                k.locks,
                k.resolutions,
                k.deprecations,
                k.unresolved_issues
            ));
        }
        if !self.ignore_overview.is_empty() {
            lines.push("npmAuditIgnoreAdvisories".into());
            for line in &self.ignore_overview {
                lines.push(format!("  {line}"));
            }
        }
        push_section_strings(&mut lines, "deprecations", &self.deprecations);
        push_section_strings(&mut lines, "unresolved issues", &self.unresolved);
        if lines.is_empty() {
            lines.push("(empty)".into());
        }
        lines
    }

    pub fn next_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if !self.report_ready {
            lines.push("— waiting for report —".into());
            return lines;
        }
        push_section_strings(&mut lines, "fixes", &self.fixes_section);
        push_section_strings(
            &mut lines,
            "suggested resolutions",
            &self.suggested_resolutions,
        );
        push_section_strings(&mut lines, "suggested ignores", &self.suggested_ignores);
        if let Some(yaml) = &self.ignore_yaml {
            for line in yaml.lines() {
                lines.push(line.to_owned());
            }
        }
        if lines.is_empty() {
            lines.push("(empty)".into());
        }
        lines
    }

    /// One-line interactive key bindings for the borderless footer.
    pub fn keyboard_hints(&self, clipboard_status: Option<&str>) -> String {
        if self.focus == PaneId::Next && !self.maximized {
            return self.copy_hints_line(clipboard_status).unwrap_or_default();
        }
        if self.focus == PaneId::Next && self.maximized {
            let mut parts = Vec::new();
            if let Some(copy) = self.copy_hints_line(clipboard_status) {
                parts.push(copy);
            }
            parts.push("[Esc] shrink".into());
            return parts.join(" · ");
        }

        if self.maximized {
            return [
                "[Tab] focus",
                "[Esc] shrink",
                "[↑↓] scroll",
                "[q] quit",
            ]
            .join(" · ");
        }

        [
            "[Tab] focus",
            "[Enter] expand",
            "[↑↓] scroll",
            "[q]/[Esc] quit",
        ]
        .join(" · ")
    }

    fn copy_hints_line(&self, clipboard_status: Option<&str>) -> Option<String> {
        let mut copy_bits = Vec::new();
        if !self.suggested_resolutions.is_empty() {
            copy_bits.push("[r] resolutions");
        }
        if self.ignore_yaml.is_some() {
            copy_bits.push("[i] ignores");
        }
        if copy_bits.is_empty() {
            return None;
        }
        let mut line = format!("Copy: {}", copy_bits.join(" · "));
        if let Some(status) = clipboard_status {
            line.push_str(&format!(" · ({status})"));
        }
        Some(line)
    }

    pub fn resolutions_clipboard(&self) -> Option<String> {
        if self.suggested_resolutions.is_empty() {
            None
        } else {
            Some(self.suggested_resolutions.join("\n"))
        }
    }

    pub fn ignores_clipboard(&self) -> Option<String> {
        self.ignore_yaml.clone()
    }

    pub fn is_live_auto(&self) -> bool {
        matches!(self.verb, Verb::Auto)
    }
}

fn push_section_strings(out: &mut Vec<String>, title: &str, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    out.push(title.to_owned());
    for line in lines {
        out.push(format!("  {line}"));
    }
}

/// Visible window into `lines` given viewport height and scroll-from-bottom.
pub fn visible_window(lines: &[String], height: usize, scroll_from_bottom: usize) -> &[String] {
    if height == 0 || lines.is_empty() {
        return &[];
    }
    if lines.len() <= height {
        return lines;
    }
    let max_scroll = lines.len() - height;
    let scroll = scroll_from_bottom.min(max_scroll);
    let end = lines.len() - scroll;
    let start = end.saturating_sub(height);
    &lines[start..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_report_omits_state_panels() {
        let mut state = UiState::new(Verb::Auto);
        state.apply(UiEvent::Activity {
            kind: ActivityKind::Yarn,
            message: "install".into(),
        });
        state.apply(UiEvent::Fix {
            message: "reset lodash".into(),
        });
        assert!(!state.report_ready);
        assert!(state.kpis.is_none());
        assert_eq!(state.fixes, vec!["reset lodash".to_string()]);
        assert!(!state.activity.is_empty());
        assert!(state.state_lines()[0].contains("waiting"));
    }

    #[test]
    fn live_fixes_record_severity_in_message() {
        let mut state = UiState::new(Verb::Auto);
        state.apply(UiEvent::Fix {
            message: "ignore 1111111  high  left-pad@<1.3.0".into(),
        });
        state.apply(UiEvent::Fix {
            message: "reset lodash  high".into(),
        });
        assert!(state.fixes.iter().any(|m| m.contains("high")));
        assert_eq!(state.fixes.len(), 2);
    }

    #[test]
    fn report_events_fill_done_regions() {
        let mut state = UiState::new(Verb::Check);
        state.apply(UiEvent::Kpis {
            dependencies: 1,
            dev_dependencies: 2,
            locks: 3,
            resolutions: 4,
            deprecations: 0,
            unresolved_issues: 1,
        });
        let line = crate::ui::format_resolution_line("pkg@^1", "1.2.3");
        state.apply(UiEvent::Section {
            title: "suggested resolutions".into(),
            lines: vec![line.clone()],
        });
        state.apply(UiEvent::IgnoreYaml {
            yaml: "npmAuditIgnoreAdvisories:\n  - \"1\"\n".into(),
        });
        state.apply(UiEvent::ReportComplete);
        assert!(state.report_ready);
        assert!(state.done);
        assert!(state.kpis.is_some());
        assert_eq!(state.resolutions_clipboard().as_deref(), Some(line.as_str()));
        assert!(
            state
                .ignores_clipboard()
                .unwrap()
                .contains("npmAuditIgnoreAdvisories")
        );
    }

    #[test]
    fn clipboard_payloads_match_section_bodies() {
        let mut state = UiState::new(Verb::Check);
        let line = crate::ui::format_resolution_line("pkg@^1", "1.2.3");
        state.apply(UiEvent::Section {
            title: "suggested resolutions".into(),
            lines: vec![line.clone()],
        });
        assert_eq!(state.resolutions_clipboard().unwrap(), line);
    }

    #[test]
    fn tab_cycles_focus() {
        let mut state = UiState::new(Verb::Auto);
        assert_eq!(state.focus, PaneId::Activity);
        state.cycle_focus();
        assert_eq!(state.focus, PaneId::Fixed);
        state.cycle_focus();
        assert_eq!(state.focus, PaneId::State);
        state.cycle_focus();
        assert_eq!(state.focus, PaneId::Next);
        state.cycle_focus();
        assert_eq!(state.focus, PaneId::Activity);
        state.cycle_focus_backward();
        assert_eq!(state.focus, PaneId::Next);
        state.cycle_focus_backward();
        assert_eq!(state.focus, PaneId::State);
    }

    #[test]
    fn scroll_and_visible_window_cover_full_history() {
        let lines: Vec<String> = (0..20).map(|i| format!("line{i}")).collect();
        let window = visible_window(&lines, 5, 0);
        assert_eq!(window.last().map(String::as_str), Some("line19"));
        let window = visible_window(&lines, 5, 15);
        assert_eq!(window.first().map(String::as_str), Some("line0"));

        let mut state = UiState::new(Verb::Auto);
        for i in 0..30 {
            state.activity.push(format!("a{i}"));
        }
        state.focus = PaneId::Activity;
        state.scroll_focused(10, 5);
        assert_eq!(state.scroll_from_bottom[0], 10);
        state.scroll_focused(100, 5);
        assert_eq!(state.scroll_from_bottom[0], 25);
    }

    #[test]
    fn copy_hints_appear_when_next_focused() {
        let mut state = UiState::new(Verb::Check);
        state.apply(UiEvent::Section {
            title: "suggested resolutions".into(),
            lines: vec![crate::ui::format_resolution_line("pkg@^1", "1.2.3")],
        });
        state.apply(UiEvent::IgnoreYaml {
            yaml: "npmAuditIgnoreAdvisories:\n  - \"1\"\n".into(),
        });
        let without = state.keyboard_hints(None);
        assert!(!without.contains("Copy:"));
        assert!(without.contains("focus"));
        state.focus = PaneId::Next;
        let with = state.keyboard_hints(None);
        assert!(with.starts_with("Copy:"));
        assert!(!with.contains("focus"));
        assert!(!with.contains("scroll"));
        assert!(!with.contains("quit"));
        assert!(with.contains("[r] resolutions"));
        assert!(with.contains("[i] ignores"));
        assert!(with.contains(" · "));

        state.maximized = true;
        let expanded = state.keyboard_hints(None);
        assert!(expanded.contains("Copy:"));
        assert!(expanded.contains("[Esc] shrink"));
    }

    #[test]
    fn enter_expand_and_esc_hints() {
        let state = UiState::new(Verb::Auto);
        assert!(state.keyboard_hints(None).contains("[Enter] expand"));
        let mut state = state;
        state.maximized = true;
        let hints = state.keyboard_hints(None);
        assert!(hints.contains("[Esc] shrink"));
        assert!(!hints.contains("[Enter] expand"));
    }

    #[test]
    fn activity_keeps_full_history() {
        let mut state = UiState::new(Verb::Auto);
        for i in 0..500 {
            state.apply(UiEvent::Activity {
                kind: ActivityKind::Info,
                message: format!("{i}"),
            });
        }
        assert_eq!(state.activity.len(), 500);
    }
}
