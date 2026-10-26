//! Shared string shapes for stdout and clipboard.

use crate::{audit::DependencyEnvironment, yarnrc::pretty_ignore_block};

/// Body line for a `package.json` resolutions / overrides fragment.
pub fn format_resolution_line(package_key: &str, version: impl std::fmt::Display) -> String {
    format!("\"{package_key}\": \"^{version}\",")
}

/// Unresolved-issue body line with trailing production/dev-only label.
pub fn format_unresolved_issue_line(
    module_name: &str,
    request: impl std::fmt::Display,
    env: DependencyEnvironment,
) -> String {
    format!("\"{module_name}@{request}\"  {}", env.label())
}

/// Suggested-ignore enrichment line with trailing production/dev-only label.
pub fn format_ignore_enrichment_line(
    id: &str,
    severity: impl std::fmt::Display,
    module_name: &str,
    vulnerable_versions: impl std::fmt::Display,
    env: DependencyEnvironment,
) -> String {
    format!(
        "{id}  {severity}  {module_name}@{vulnerable_versions}  {}",
        env.label()
    )
}

/// Sort + dedup section body lines (same as historical `print_section`).
pub fn format_section_lines(mut lines: Vec<String>) -> Vec<String> {
    lines.sort_unstable();
    lines.dedup();
    lines
}

/// Paste-ready ignore YAML (delegates to yarnrc helper).
pub fn format_ignore_yaml(ids: &[String]) -> String {
    pretty_ignore_block(ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_line_matches_historical_shape() {
        assert_eq!(
            format_resolution_line("lodash@^4", "4.17.21"),
            "\"lodash@^4\": \"^4.17.21\","
        );
    }

    #[test]
    fn ignore_yaml_matches_pretty_ignore_block() {
        let ids = vec!["123".into(), "GHSA-x".into()];
        assert_eq!(format_ignore_yaml(&ids), pretty_ignore_block(&ids));
    }

    #[test]
    fn section_lines_sort_and_dedup() {
        assert_eq!(
            format_section_lines(vec!["b".into(), "a".into(), "a".into()]),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn unresolved_issue_line_keeps_prefix_and_appends_label() {
        use crate::audit::DependencyEnvironment;
        assert_eq!(
            format_unresolved_issue_line("lodash", "^4", DependencyEnvironment::Prod),
            "\"lodash@^4\"  prod"
        );
        assert_eq!(
            format_unresolved_issue_line("eslint", "^8", DependencyEnvironment::DevOnly),
            "\"eslint@^8\"  dev-only"
        );
    }

    #[test]
    fn ignore_enrichment_line_appends_label_without_touching_yaml() {
        use crate::audit::DependencyEnvironment;
        assert_eq!(
            format_ignore_enrichment_line(
                "1090865",
                "high",
                "left-pad",
                "<1.0.0",
                DependencyEnvironment::DevOnly
            ),
            "1090865  high  left-pad@<1.0.0  dev-only"
        );
        let yaml = format_ignore_yaml(&["1090865".into()]);
        assert!(!yaml.contains("dev-only"));
        assert!(!yaml.contains("prod"));
        assert!(yaml.contains("1090865"));
    }
}
