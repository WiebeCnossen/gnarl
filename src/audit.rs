use std::str::FromStr;

use nodejs_semver::{Range, Version};
use serde::Deserialize;

use crate::{Package, parse};

#[derive(Deserialize)]
struct AuditDto {
    value: String,
    children: AuditDtoChildren,
}

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "moderate")]
    Moderate,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

impl FromStr for Severity {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "info" => Severity::Info,
            "low" => Severity::Low,
            "moderate" => Severity::Moderate,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => return Err(format!("invalid severity: {}", s).into()),
        })
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Severity::Info => "info",
                Severity::Low => "low",
                Severity::Moderate => "moderate",
                Severity::High => "high",
                Severity::Critical => "critical",
            }
        )
    }
}

impl Severity {
    pub fn label(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Low => "low",
            Severity::Moderate => "moderate",
            Severity::High => "high",
            Severity::Critical => "critical",
        }
    }

    /// Yarn `--severity` semantics: include this level and anything more severe.
    pub fn meets_threshold(self, threshold: Severity) -> bool {
        self >= threshold
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct AuditDtoChildren {
    #[serde(rename = "ID")]
    id: serde_json::Value, // Accepts either integer or string
    #[serde(rename = "Severity")]
    severity: Severity,
    #[serde(rename = "Vulnerable Versions")]
    vulnerable_versions: String,
    #[serde(rename = "Tree Versions")]
    tree_versions: Vec<String>,
    #[serde(rename = "Dependents")]
    dependents: Vec<String>,
}

pub struct Advisory {
    id: String,
    module_name: String,
    severity: Severity,
    vulnerable_versions: Range,
    tree_versions: Vec<Version>,
    #[allow(unused)]
    dependents: Vec<Package>,
    root_name: Option<String>,
}

fn normalize_advisory_id(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n
            .as_u64()
            .map(|u| u.to_string())
            .or_else(|| n.as_i64().map(|i| i.to_string()))
            .unwrap_or_else(|| n.to_string()),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string().trim_matches('"').to_owned(),
    }
}

impl TryFrom<AuditDto> for Advisory {
    type Error = crate::Error;
    fn try_from(dto: AuditDto) -> Result<Self, Self::Error> {
        Ok(Self {
            id: normalize_advisory_id(&dto.children.id),
            module_name: dto.value,
            severity: dto.children.severity,
            vulnerable_versions: parse::parse_range(&dto.children.vulnerable_versions)?,
            tree_versions: dto
                .children
                .tree_versions
                .iter()
                .map(|v| parse::parse_version(v))
                .collect::<Result<Vec<Version>, crate::Error>>()?,
            dependents: dto
                .children
                .dependents
                .iter()
                .map(|d| Package::try_from(d.to_owned()))
                .collect::<Result<Vec<Package>, crate::Error>>()?,
            root_name: None,
        })
    }
}

impl Advisory {
    pub fn new(
        id: String,
        module_name: String,
        severity: Severity,
        vulnerable_versions: Range,
        tree_versions: Vec<Version>,
        dependents: Vec<Package>,
        root_name: Option<String>,
    ) -> Self {
        Self {
            id,
            module_name,
            severity,
            vulnerable_versions,
            tree_versions,
            dependents,
            root_name,
        }
    }

    pub fn parse(s: &str) -> Result<Self, crate::Error> {
        let dto: AuditDto = serde_json::from_str(s)?;
        Advisory::try_from(dto)
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn module_name(&self) -> &str {
        &self.module_name
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn vulnerable_versions(&self) -> &Range {
        &self.vulnerable_versions
    }

    pub fn tree_versions(&self) -> &[Version] {
        &self.tree_versions
    }

    #[allow(unused)]
    pub fn dependents(&self) -> &[Package] {
        &self.dependents
    }

    pub fn root_name(&self) -> Option<&str> {
        self.root_name.as_deref()
    }

    pub fn is_deprecation(&self) -> bool {
        self.id().contains(" (deprecation)")
    }

    pub fn label(&self) -> &'static str {
        if self.is_deprecation() {
            "deprecation"
        } else {
            self.severity.label()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering_matches_yarn_threshold() {
        assert!(Severity::Info < Severity::Low);
        assert!(Severity::Low < Severity::Moderate);
        assert!(Severity::Moderate < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn meets_threshold_includes_equal_and_higher() {
        assert!(Severity::High.meets_threshold(Severity::High));
        assert!(Severity::Critical.meets_threshold(Severity::High));
        assert!(!Severity::Moderate.meets_threshold(Severity::High));
        assert!(Severity::Info.meets_threshold(Severity::Info));
    }

    fn sample_audit_line(id_json: &str) -> String {
        format!(
            r#"{{"value":"left-pad","children":{{"ID":{id_json},"Issue":"x","Severity":"high","Vulnerable Versions":"<1.0.0","Tree Versions":["0.0.1"],"Dependents":["pkg@npm:1.0.0"]}}}}"#
        )
    }

    #[test]
    fn advisory_id_accepts_integer_or_string_and_matches() {
        let from_int = Advisory::parse(&sample_audit_line("1090865")).unwrap();
        let from_str = Advisory::parse(&sample_audit_line(r#""1090865""#)).unwrap();
        assert_eq!(from_int.id(), "1090865");
        assert_eq!(from_str.id(), "1090865");
        assert_eq!(from_int.id(), from_str.id());
        // Same canonical form yarnrc uses for unquoted integer ignore entries.
        assert_eq!(
            normalize_advisory_id(&serde_json::json!(1090865)),
            "1090865"
        );
    }
}
