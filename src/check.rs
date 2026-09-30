use std::collections::BTreeMap;

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

    pub fn emit(&self, reporter: &dyn crate::ui::Reporter) {
        reporter.emit(crate::ui::UiEvent::Kpis {
            dependencies: self.dependencies,
            dev_dependencies: self.dev_dependencies,
            locks: self.locks,
            resolutions: self.resolutions,
            deprecations: self.deprecations,
            unresolved_issues: self.unresolved_issues,
        });
    }
}

#[allow(unused)]
pub struct Check {
    pub resolutions: BTreeMap<String, String>,
    pub deprecations: BTreeMap<String, String>,
    pub resolution_suggestions: BTreeMap<String, String>,
    pub upgrade_suggestions: BTreeMap<String, String>,
    pub unresolved_issues: BTreeMap<String, String>,
}
