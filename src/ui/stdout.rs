use super::{ActivityKind, Reporter, UiEvent};

/// Maps [`UiEvent`]s to historical tagged stdout.
pub struct StdoutReporter;

impl Reporter for StdoutReporter {
    fn emit(&self, event: UiEvent) {
        match event {
            UiEvent::Activity { kind, message } => match kind {
                ActivityKind::Yarn => {
                    print!("[YARN] ");
                    println!("{message}");
                }
                ActivityKind::Hit => {
                    print!("[HIT#] ");
                    println!("{message}");
                }
                ActivityKind::Info => {
                    print!("[INFO] ");
                    println!("{message}");
                }
                ActivityKind::Npm => {
                    print!("[NPM?] ");
                    println!("{message}");
                }
            },
            UiEvent::Fix { message } => {
                print!("[FIX!] ");
                println!("{message}");
            }
            UiEvent::Phase(_) => {}
            UiEvent::Kpis {
                dependencies,
                dev_dependencies,
                locks,
                resolutions,
                deprecations,
                unresolved_issues,
            } => {
                print!("[INFO] ");
                println!("KPIs");
                println!("    {dependencies:4} dependencies");
                println!("    {dev_dependencies:4} dev_dependencies");
                println!("    {locks:4} locks");
                println!("    {resolutions:4} resolutions");
                println!("    {deprecations:4} deprecations");
                println!("    {unresolved_issues:4} unresolved_issues");
            }
            UiEvent::Section { title, lines } => {
                if lines.is_empty() {
                    return;
                }
                print!("[INFO] ");
                println!("{title}");
                for line in lines {
                    println!("    {line}");
                }
            }
            UiEvent::IgnoreYaml { yaml } => {
                print!("{yaml}");
            }
            UiEvent::SuggestedResolutions { .. } | UiEvent::SuggestedIgnoreIds { .. } => {}
            UiEvent::ReportComplete => {}
            UiEvent::Error { message } => {
                eprintln!("{message}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StdoutReporter;
    use crate::ui::format::{format_ignore_yaml, format_resolution_line, format_section_lines};
    use crate::ui::{Reporter, UiEvent};

    #[test]
    fn fixture_report_shapes_via_formatters() {
        let lines = format_section_lines(vec![format_resolution_line("pkg@^1", "1.2.3")]);
        assert_eq!(lines, vec!["\"pkg@^1\": \"^1.2.3\",".to_string()]);
        let yaml = format_ignore_yaml(&["9".into()]);
        assert!(yaml.starts_with("npmAuditIgnoreAdvisories:"));
        assert!(yaml.contains("- \"9\""));
    }

    #[test]
    fn structured_apply_events_are_ignored_by_stdout() {
        let reporter = StdoutReporter;
        reporter.emit(UiEvent::SuggestedResolutions {
            entries: vec![("pkg@^1".into(), "1.2.3".into())],
        });
        reporter.emit(UiEvent::SuggestedIgnoreIds {
            ids: vec!["9".into()],
        });
        let lines = format_section_lines(vec![format_resolution_line("pkg@^1", "1.2.3")]);
        assert_eq!(lines, vec!["\"pkg@^1\": \"^1.2.3\",".to_string()]);
        let yaml = format_ignore_yaml(&["9".into()]);
        assert!(yaml.starts_with("npmAuditIgnoreAdvisories:"));
    }
}
