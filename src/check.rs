use std::collections::BTreeMap;

use crate::{out_indent, out_info};

#[allow(unused)]
pub struct Check {
    pub resolutions: BTreeMap<String, String>,
    pub deprecations: BTreeMap<String, String>,
    pub resolution_suggestions: BTreeMap<String, String>,
    pub upgrade_suggestions: BTreeMap<String, String>,
    pub unresolved_issues: BTreeMap<String, String>,
}

pub struct Kpis {
    dependencies: usize,
    dev_dependencies: usize,
    locks: usize,
    resolutions: usize,
    deprecations: usize,
    unresolved_issues: usize,
}

impl Kpis {
    pub fn new(
        dependencies: usize,
        dev_dependencies: usize,
        locks: usize,
        resolutions: usize,
        deprecations: usize,
        unresolved_issues: usize,
    ) -> Kpis {
        Kpis {
            dependencies,
            dev_dependencies,
            locks,
            resolutions,
            deprecations,
            unresolved_issues,
        }
    }

    pub fn print(&self) {
        out_info!("KPIs");
        out_indent!("{:4} {}", self.dependencies, "dependencies");
        out_indent!("{:4} {}", self.dev_dependencies, "dev_dependencies");
        out_indent!("{:4} {}", self.locks, "locks");
        out_indent!("{:4} {}", self.resolutions, "resolutions");
        out_indent!("{:4} {}", self.deprecations, "deprecations");
        out_indent!("{:4} {}", self.unresolved_issues, "unresolved_issues");
    }
}
