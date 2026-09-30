//! Shared string shapes for stdout and clipboard.

use crate::yarnrc::pretty_ignore_block;

/// Body line for a `package.json` resolutions / overrides fragment.
pub fn format_resolution_line(package_key: &str, version: impl std::fmt::Display) -> String {
    format!("\"{package_key}\": \"^{version}\",")
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
}
