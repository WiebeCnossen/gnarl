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
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "moderate")]
    Moderate,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "critical")]
    Critical,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
struct AuditDtoChildren {
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
    module_name: String,
    severity: Severity,
    vulnerable_versions: Range,
    tree_versions: Vec<Version>,
    dependents: Vec<Package>,
}

impl TryFrom<AuditDto> for Advisory {
    type Error = crate::Error;
    fn try_from(dto: AuditDto) -> Result<Self, Self::Error> {
        Ok(Self {
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
        })
    }
}

impl Advisory {
    pub fn new(
        module_name: String,
        severity: Severity,
        vulnerable_versions: Range,
        tree_versions: Vec<Version>,
        dependents: Vec<Package>,
    ) -> Self {
        Self {
            module_name,
            severity,
            vulnerable_versions,
            tree_versions,
            dependents,
        }
    }

    pub fn parse(s: &str) -> Result<Self, crate::Error> {
        let dto: AuditDto = serde_json::from_str(s)?;
        Advisory::try_from(dto)
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

    pub fn dependents(&self) -> &[Package] {
        &self.dependents
    }
}
