use std::collections::{BTreeMap, HashMap, HashSet};

use nodejs_semver::{OutsideDirection, Range, Version};

use crate::{
    Error, RunStatus,
    audit::{Advisory, Severity},
    check::Kpis,
    cmd::Options,
    npm::{Npm, Packument},
    package::Dependency,
    parse,
    status,
    ui::{
        ActivityKind, SharedReporter, UiEvent, format_ignore_yaml, format_resolution_line,
        format_section_lines, stdout_reporter,
    },
    yarn::Yarn,
};

pub struct Gnarl {
    options: Options,
    npm: Npm,
    reset: HashSet<String>,
    /// Printed `{package} blocked by {other}@{version}` keys for this process run.
    blocked_by: HashSet<String>,
    reporter: SharedReporter,
}

struct Classification {
    deprecations: Vec<String>,
    fixes: BTreeMap<String, SuggestedFix>,
    resolutions: BTreeMap<String, SuggestedFix>,
    errors: Vec<String>,
    ignore_suggestions: BTreeMap<String, IgnoreSuggestion>,
    lock_len: usize,
}

struct SuggestedFix {
    version: Version,
}

impl Gnarl {
    pub fn new(options: Options) -> Result<Self, Error> {
        Self::with_reporter(options, stdout_reporter())
    }

    pub fn with_reporter(options: Options, reporter: SharedReporter) -> Result<Self, Error> {
        Ok(Self {
            options,
            npm: Npm::with_reporter(reporter.clone())?,
            reset: HashSet::new(),
            blocked_by: HashSet::new(),
            reporter,
        })
    }

    fn yarn(&self) -> Result<Yarn, Error> {
        Yarn::with_reporter(self.options.severity(), self.reporter.clone())
    }

    pub fn check(&mut self) -> Result<RunStatus, Error> {
        self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Report));
        let mut yarn = self.yarn()?;
        let advisories = yarn.audit()?;
        let class = self.classify(&mut yarn, advisories, true)?;
        self.emit_report(&yarn, class)?;
        Ok(RunStatus::ok())
    }

    fn classify(
        &mut self,
        yarn: &mut Yarn,
        advisories: Vec<Advisory>,
        emit_hits: bool,
    ) -> Result<Classification, Error> {
        let mut deprecations = vec![];
        let mut fixes = BTreeMap::new();
        let mut resolutions = BTreeMap::new();
        let mut errors = vec![];
        let mut ignore_suggestions = BTreeMap::new();
        if emit_hits {
            for advisory in &advisories {
                self.reporter.emit(UiEvent::Activity {
                    kind: ActivityKind::Hit,
                    message: format!(
                        "{}: {}@{}",
                        advisory.label(),
                        advisory.module_name(),
                        advisory.vulnerable_versions()
                    ),
                });
            }
        }

        let lock_len = {
            let locks = yarn.locks()?;
            for advisory in advisories {
                if advisory.is_deprecation() {
                    deprecations.push(format!(
                        "\"{}@{}\"",
                        advisory.module_name(),
                        advisory
                            .tree_versions()
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                    continue;
                }

                self.npm.retrieve_packument(advisory.module_name())?;
                let packument = self.npm.packument(advisory.module_name())?;
                let yarn_resolutions = locks.for_package(advisory.module_name());
                for tree_version in advisory.tree_versions() {
                    let resolution = yarn_resolutions
                        .iter()
                        .find(|resolution| {
                            resolution
                                .package()
                                .version()
                                .map(|v| v.to_string() == tree_version.to_string())
                                .unwrap_or(false)
                        })
                        .ok_or_else(|| format!("Resolution for {} not found", tree_version))?;
                    for request in resolution.requests() {
                        let original_request = resolution.original(request)?;
                        if let Some(fix) = get_fix(
                            packument,
                            advisory.vulnerable_versions(),
                            advisory.tree_versions().last().unwrap(),
                            request,
                        ) {
                            add_fix(
                                &mut fixes,
                                advisory.module_name(),
                                original_request,
                                fix,
                            );
                        } else if let Some(fix) =
                            get_resolution(packument, advisory.vulnerable_versions(), request)
                        {
                            add_fix(
                                &mut resolutions,
                                advisory.module_name(),
                                original_request,
                                fix,
                            );
                            record_ignore_suggestion(&mut ignore_suggestions, &advisory);
                        } else {
                            errors.push(format!(
                                "\"{}@{}\"",
                                advisory.module_name(),
                                original_request
                            ));
                            record_ignore_suggestion(&mut ignore_suggestions, &advisory);
                        }
                    }
                }
            }
            locks.len()
        };

        fixes.retain(|key, _| !resolutions.contains_key(key));

        Ok(Classification {
            deprecations,
            fixes,
            resolutions,
            errors,
            ignore_suggestions,
            lock_len,
        })
    }

    fn emit_report(&mut self, yarn: &Yarn, class: Classification) -> Result<(), Error> {
        Kpis::new(
            yarn.len_dependencies(),
            yarn.len_dev_dependencies(),
            class.lock_len,
            yarn.len_resolutions(),
            class.deprecations.len(),
            class.fixes.len() + class.resolutions.len() + class.errors.len(),
        )
        .emit(&*self.reporter);

        self.print_ignore_overview(yarn)?;

        emit_section(&*self.reporter, "deprecations", class.deprecations);
        emit_section(
            &*self.reporter,
            "fixes",
            class
                .fixes
                .iter()
                .map(|(k, v)| format_resolution_line(k, &v.version))
                .collect(),
        );
        emit_section(
            &*self.reporter,
            "suggested resolutions",
            class
                .resolutions
                .iter()
                .map(|(k, v)| format_resolution_line(k, &v.version))
                .collect(),
        );
        emit_section(&*self.reporter, "unresolved issues", class.errors);
        self.print_suggested_ignores(yarn, class.ignore_suggestions)?;
        self.reporter.emit(UiEvent::ReportComplete);
        self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Done));
        Ok(())
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<bool, Error> {
        self.reset
            .extend(packages.iter().map(|p| p.as_ref().to_string()));
        let dirty = self.yarn()?.locks()?.reset(packages)?;
        Ok(dirty && !self.options.no_install())
    }

    pub fn auto(&mut self) -> Result<RunStatus, Error> {
        let _: () = loop {
            let mut yarn = self.yarn()?;
            self.reporter
                .emit(UiEvent::Phase(crate::ui::Phase::Install));
            yarn.install()?;
            self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Dedupe));
            yarn.dedupe()?;

            let mut dirty = false;
            let mut resets = vec![];
            let mut reset_severities = HashMap::new();
            self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Audit));
            let mut advisories = yarn.audit()?;
            self.reporter.emit(UiEvent::Activity {
                kind: ActivityKind::Info,
                message: format!("{} advisories", advisories.len()),
            });
            let mut done = HashSet::new();

            self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Fix));
            while let Some(advisory) = advisories.pop() {
                if done.insert(format!("{} {}", advisory.id(), advisory.module_name()))
                    && self.fix(&mut yarn, &advisory, &mut advisories)?
                {
                    dirty = true;
                    note_reset_severity(
                        &mut reset_severities,
                        advisory.module_name(),
                        advisory.severity(),
                    );
                    resets.push(advisory.module_name().to_owned());
                }
            }

            if !dirty || self.options.no_install() {
                break;
            }

            if !resets.is_empty() {
                yarn.locks()?
                    .reset_with_severities(&resets, &reset_severities)?;
                self.reset.extend(resets);
            }
        };

        self.reporter
            .emit(UiEvent::Phase(crate::ui::Phase::Hygiene));
        let mut yarn = self.yarn()?;
        let resolutions_dirty = yarn.reset_resolutions()?;
        let ignore_resets = self.reset_ignored_advisories(&mut yarn)?;
        if (resolutions_dirty || !ignore_resets.is_empty()) && !self.options.no_install() {
            let yarn = self.yarn()?;
            yarn.install()?;
            yarn.dedupe()?;
        }

        let policy = if self.options.auto_ignore() {
            self.persist_suggested_ignores()?
        } else {
            RunStatus::ok()
        };
        self.check()?;
        Ok(policy)
    }

    fn persist_suggested_ignores(&mut self) -> Result<RunStatus, Error> {
        let mut yarn = self.yarn()?;
        let advisories = yarn.audit()?;
        let class = self.classify(&mut yarn, advisories, false)?;
        let mut yarnrc = yarn.yarnrc()?;
        let existing: HashSet<String> = yarnrc.npm_audit_ignore_advisories().into_iter().collect();
        let to_write = new_ignore_suggestions(&class.ignore_suggestions, &existing);
        if to_write.is_empty() {
            return Ok(RunStatus::ok());
        }

        let mut max_sev: Option<Severity> = None;
        let ids: Vec<String> = to_write.iter().map(|s| s.id.clone()).collect();
        for suggestion in &to_write {
            max_sev = Some(match max_sev {
                None => suggestion.severity,
                Some(current) => current.max(suggestion.severity),
            });
            self.reporter.emit(UiEvent::Fix {
                message: ignore_fix_message(suggestion),
            });
        }
        yarnrc.merge_npm_audit_ignore_advisories(&ids);
        yarnrc.save()?;
        Ok(status::auto_ignore_status(true, max_sev))
    }

    fn print_ignore_overview(&mut self, yarn: &Yarn) -> Result<(), Error> {
        let yarnrc = yarn.yarnrc()?;
        let ignores = yarnrc.npm_audit_ignore_advisories();
        if ignores.is_empty() {
            return Ok(());
        }

        let unfiltered = yarn.audit_unfiltered()?;
        let by_id: HashMap<&str, &Advisory> = unfiltered
            .iter()
            .map(|advisory| (advisory.id(), advisory))
            .collect();

        let mut lines = Vec::new();
        for id in &ignores {
            match by_id.get(id.as_str()) {
                Some(advisory) => lines.push(format!(
                    "{}  {}  {}@{}",
                    id,
                    advisory.severity(),
                    advisory.module_name(),
                    advisory.vulnerable_versions()
                )),
                None => lines.push(format!("{}  unknown", id)),
            }
        }

        emit_section(&*self.reporter, "npmAuditIgnoreAdvisories", lines);
        Ok(())
    }

    fn print_suggested_ignores(
        &self,
        yarn: &Yarn,
        suggestions: BTreeMap<String, IgnoreSuggestion>,
    ) -> Result<(), Error> {
        let existing: HashSet<String> = yarn
            .yarnrc()?
            .npm_audit_ignore_advisories()
            .into_iter()
            .collect();

        let to_write = new_ignore_suggestions(&suggestions, &existing);
        if to_write.is_empty() {
            return Ok(());
        }

        let lines: Vec<String> = to_write
            .iter()
            .map(|s| {
                format!(
                    "{}  {}  {}@{}",
                    s.id, s.severity, s.module_name, s.vulnerable_versions
                )
            })
            .collect();
        let ids: Vec<String> = to_write.iter().map(|s| s.id.clone()).collect();

        emit_section(&*self.reporter, "suggested ignores", lines);
        self.reporter.emit(UiEvent::IgnoreYaml {
            yaml: format_ignore_yaml(&ids),
        });
        Ok(())
    }

    fn reset_ignored_advisories(&mut self, yarn: &mut Yarn) -> Result<Vec<String>, Error> {
        let mut yarnrc = yarn.yarnrc()?;
        let ignores = yarnrc.npm_audit_ignore_advisories();
        if ignores.is_empty() {
            return Ok(Vec::new());
        }

        let unfiltered = yarn.audit_unfiltered()?;
        let by_id: HashMap<String, Advisory> = unfiltered
            .into_iter()
            .map(|advisory| (advisory.id().to_owned(), advisory))
            .collect();

        let mut yarnrc_dirty = false;
        let mut resets = Vec::new();
        let mut reset_severities = HashMap::new();

        for id in &ignores {
            match by_id.get(id) {
                None => {
                    self.reporter.emit(UiEvent::Fix {
                        message: format!("drop orphan ignore {id}"),
                    });
                    if yarnrc.remove_npm_audit_ignore_advisory(id) {
                        yarnrc_dirty = true;
                    }
                }
                Some(advisory) if advisory.is_deprecation() => {}
                Some(advisory) => {
                    if self.within_range_resettable(yarn, advisory)? {
                        self.reporter.emit(UiEvent::Fix {
                            message: drop_within_range_ignore_message(id, advisory.severity()),
                        });
                        if yarnrc.remove_npm_audit_ignore_advisory(id) {
                            yarnrc_dirty = true;
                        }
                        note_reset_severity(
                            &mut reset_severities,
                            advisory.module_name(),
                            advisory.severity(),
                        );
                        resets.push(advisory.module_name().to_owned());
                    }
                }
            }
        }

        if yarnrc_dirty {
            yarnrc.save()?;
        }

        if !resets.is_empty() {
            yarn.locks()?
                .reset_with_severities(&resets, &reset_severities)?;
            self.reset.extend(resets.iter().cloned());
        }

        Ok(resets)
    }

    fn within_range_resettable(
        &mut self,
        yarn: &mut Yarn,
        advisory: &Advisory,
    ) -> Result<bool, Error> {
        self.npm.retrieve_packument(advisory.module_name())?;
        let packument = self.npm.packument(advisory.module_name()).cloned()?;
        let mut fixable = false;
        let mut blocked = false;
        let dependents = yarn.locks()?.dependents(advisory.module_name()).to_vec();
        for dependent in &dependents {
            let tree_version = match advisory
                .tree_versions()
                .iter()
                .rfind(|v| v.satisfies(dependent.request()))
            {
                Some(v) => v,
                None => continue,
            };
            if has_fix(
                &packument,
                advisory.vulnerable_versions(),
                tree_version,
                dependent.request(),
            ) {
                fixable = true;
            } else {
                blocked = true;
            }
        }

        Ok(fixable && !blocked && !self.reset.contains(advisory.module_name()))
    }

    fn fix(
        &mut self,
        yarn: &mut Yarn,
        advisory: &Advisory,
        advisories: &mut Vec<Advisory>,
    ) -> Result<bool, Error> {
        self.npm.retrieve_packument(advisory.module_name())?;
        let packument = self.npm.packument(advisory.module_name()).cloned()?;
        let mut fixable = false;
        let dependents = yarn.locks()?.dependents(advisory.module_name()).to_vec();
        for dependent in &dependents {
            let tree_version = match advisory
                .tree_versions()
                .iter()
                .rfind(|v| v.satisfies(dependent.request()))
            {
                Some(v) => v,
                None => continue,
            };
            if has_fix(
                &packument,
                advisory.vulnerable_versions(),
                tree_version,
                dependent.request(),
            ) {
                fixable = true;
                continue;
            }

            // Blocked range: escalate via parent advisory; still reset if another
            // range is within-range fixable (partial win).
            if dependent.source() == "npm" {
                advisories.push(
                    self.create_advisory(
                        yarn,
                        advisory,
                        dependent,
                        advisory
                            .root_name()
                            .unwrap_or(advisory.module_name())
                            .to_owned(),
                    )?,
                );
            } else {
                let message = format!(
                    "{} blocked by {}@{}",
                    advisory.root_name().unwrap_or(advisory.module_name()),
                    advisory.module_name(),
                    tree_version
                );
                if self.blocked_by.insert(message.clone()) {
                    self.reporter.emit(UiEvent::Activity {
                        kind: ActivityKind::Info,
                        message,
                    });
                }
            }
        }

        if fixable && !self.reset.contains(advisory.module_name()) {
            return Ok(true);
        }

        if advisory.root_name().is_none()
            && !advisory.is_deprecation()
            && !has_fix(
                &packument,
                advisory.vulnerable_versions(),
                advisory.tree_versions().last().unwrap(),
                &parse::parse_range("*")?,
            )
        {
            self.reporter.emit(UiEvent::Activity {
                kind: ActivityKind::Info,
                message: format!(
                    "{}@{} has no fix",
                    advisory.module_name(),
                    advisory.vulnerable_versions()
                ),
            });
        }

        Ok(false)
    }

    fn create_advisory(
        &mut self,
        yarn: &mut Yarn,
        advisory: &Advisory,
        dependent: &Dependency,
        root_name: String,
    ) -> Result<Advisory, Error> {
        self.npm.retrieve_packument(dependent.name())?;
        let packument = self.npm.packument(dependent.name())?;
        let tree_versions = yarn
            .locks()?
            .for_package(dependent.name())
            .iter()
            .flat_map(|resolution| resolution.package().version())
            .cloned()
            .collect();
        let vulnerable_versions = packument
            .versions()
            .filter(|version| {
                let version = packument.version(version).unwrap();
                version.dependencies().any(|dependency| {
                    dependency == advisory.module_name()
                        && version
                            .dependency(dependency)
                            .unwrap()
                            .difference(advisory.vulnerable_versions())
                            .is_none()
                })
            })
            .map(|version| version.to_string())
            .collect::<Vec<_>>()
            .join(" || ");
        Ok(Advisory::new(
            advisory.id().to_owned(),
            dependent.name().to_owned(),
            advisory.severity(),
            parse::parse_range(&vulnerable_versions)?,
            tree_versions,
            vec![],
            Some(root_name),
        ))
    }
}

fn has_fix(
    packument: &Packument,
    vulnerable_versions: &Range,
    tree_version: &Version,
    request: &Range,
) -> bool {
    get_fix(packument, vulnerable_versions, tree_version, request).is_some()
}

fn get_fix<'a>(
    packument: &'a Packument,
    vulnerable_versions: &Range,
    tree_version: &Version,
    request: &Range,
) -> Option<&'a Version> {
    packument.versions().find(|version| {
        tree_version.lt(version)
            && request.satisfies(version)
            && !vulnerable_versions.satisfies(version)
    })
}

fn get_resolution<'a>(
    packument: &'a Packument,
    vulnerable_versions: &Range,
    request: &Range,
) -> Option<&'a Version> {
    packument.versions().find(|version| {
        request.outside(version, OutsideDirection::Higher, false) == Ok(true)
            && !vulnerable_versions.satisfies(version)
    })
}

fn emit_section(reporter: &dyn crate::ui::Reporter, title: &str, lines: Vec<String>) {
    let lines = format_section_lines(lines);
    if lines.is_empty() {
        return;
    }
    reporter.emit(UiEvent::Section {
        title: title.to_owned(),
        lines,
    });
}

struct IgnoreSuggestion {
    id: String,
    severity: Severity,
    module_name: String,
    vulnerable_versions: String,
}

fn ignore_fix_message(suggestion: &IgnoreSuggestion) -> String {
    format!(
        "ignore {}  {}  {}@{}",
        suggestion.id, suggestion.severity, suggestion.module_name, suggestion.vulnerable_versions
    )
}

fn drop_within_range_ignore_message(id: &str, severity: Severity) -> String {
    format!("drop ignore {id} (within-range fix)  {severity}")
}

fn new_ignore_suggestions<'a>(
    suggestions: &'a BTreeMap<String, IgnoreSuggestion>,
    existing: &HashSet<String>,
) -> Vec<&'a IgnoreSuggestion> {
    suggestions
        .values()
        .filter(|s| !existing.contains(&s.id))
        .collect()
}

fn note_reset_severity(map: &mut HashMap<String, Severity>, package: &str, severity: Severity) {
    map.entry(package.to_owned())
        .and_modify(|current| {
            if severity > *current {
                *current = severity;
            }
        })
        .or_insert(severity);
}

fn record_ignore_suggestion(
    map: &mut BTreeMap<String, IgnoreSuggestion>,
    advisory: &Advisory,
) {
    map.entry(advisory.id().to_owned()).or_insert_with(|| IgnoreSuggestion {
        id: advisory.id().to_owned(),
        severity: advisory.severity(),
        module_name: advisory.module_name().to_owned(),
        vulnerable_versions: advisory.vulnerable_versions().to_string(),
    });
}

fn add_fix(
    map: &mut BTreeMap<String, SuggestedFix>,
    package: &str,
    request: &str,
    resolution: &Version,
) {
    map.entry(format!("{}@{}", package, request))
        .and_modify(|v| {
            if v.version.lt(resolution) {
                v.version = resolution.to_owned();
            }
        })
        .or_insert(SuggestedFix {
            version: resolution.to_owned(),
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::auto_ignore_status;

    fn suggestion(id: &str, severity: Severity) -> IgnoreSuggestion {
        IgnoreSuggestion {
            id: id.to_owned(),
            severity,
            module_name: "left-pad".to_owned(),
            vulnerable_versions: "<1.3.0".to_owned(),
        }
    }

    #[test]
    fn no_candidates_means_policy_zero_and_nothing_to_write() {
        let suggestions = BTreeMap::new();
        let existing = HashSet::new();
        assert!(new_ignore_suggestions(&suggestions, &existing).is_empty());
        assert_eq!(auto_ignore_status(true, None).policy_exit(), 0);
    }

    #[test]
    fn high_and_low_new_ignores_map_to_exit_13() {
        let mut suggestions = BTreeMap::new();
        suggestions.insert("1".into(), suggestion("1", Severity::High));
        suggestions.insert("2".into(), suggestion("2", Severity::Low));
        let existing = HashSet::new();
        let to_write = new_ignore_suggestions(&suggestions, &existing);
        let max = to_write.iter().map(|s| s.severity).max();
        assert_eq!(
            auto_ignore_status(true, max).policy_exit(),
            13
        );
        assert_eq!(
            auto_ignore_status(false, max).policy_exit(),
            0
        );
    }

    #[test]
    fn written_ids_are_omitted_from_later_suggestions() {
        let mut suggestions = BTreeMap::new();
        suggestions.insert("1111111".into(), suggestion("1111111", Severity::High));
        let mut existing = HashSet::new();
        existing.insert("1111111".into());
        assert!(new_ignore_suggestions(&suggestions, &existing).is_empty());
    }

    #[test]
    fn ignore_fix_message_includes_id_and_severity() {
        let s = suggestion("1111111", Severity::High);
        let msg = ignore_fix_message(&s);
        assert!(msg.contains("1111111"));
        assert!(msg.contains("high"));
    }

    #[test]
    fn drop_within_range_message_includes_severity_and_does_not_set_policy() {
        let msg = drop_within_range_ignore_message("1111111", Severity::Critical);
        assert!(msg.contains("critical"));
        assert_eq!(
            auto_ignore_status(true, None).policy_exit(),
            0
        );
    }

    #[test]
    fn reset_severity_map_uses_maximum() {
        let mut map = HashMap::new();
        note_reset_severity(&mut map, "lodash", Severity::Moderate);
        note_reset_severity(&mut map, "lodash", Severity::Critical);
        assert_eq!(map.get("lodash"), Some(&Severity::Critical));
    }
}
