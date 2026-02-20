use std::str::FromStr;

use nodejs_semver::{Range, Version};
use serde::Deserialize;

use crate::{Package, parse};

#[derive(Deserialize)]
struct AuditDto {
    value: String,
    children: AuditDtoChildren,
}

#[derive(Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
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

impl TryFrom<AuditDto> for Advisory {
    type Error = crate::Error;
    fn try_from(dto: AuditDto) -> Result<Self, Self::Error> {
        Ok(Self {
            id: dto.children.id.to_string(),
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
}
