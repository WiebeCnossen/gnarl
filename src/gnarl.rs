use std::collections::{BTreeMap, HashMap, HashSet};

use nodejs_semver::{OutsideDirection, Range, Version};

use crate::{
    Error,
    audit::Advisory,
    check::Kpis,
    cmd::Options,
    npm::{Npm, Packument},
    package::Dependency,
    parse,
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

    pub fn check(&mut self) -> Result<(), Error> {
        self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Report));
        let mut yarn = self.yarn()?;

        let advisories = yarn.audit()?;
        let mut deprecations = vec![];
        let mut fixes = BTreeMap::new();
        let mut resolutions = BTreeMap::new();
        let mut errors = vec![];
        let mut ignore_suggestions = BTreeMap::new();
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

        Kpis::new(
            yarn.len_dependencies(),
            yarn.len_dev_dependencies(),
            lock_len,
            yarn.len_resolutions(),
            deprecations.len(),
            fixes.len() + resolutions.len() + errors.len(),
        )
        .emit(&*self.reporter);

        self.print_ignore_overview(&yarn)?;

        emit_section(&*self.reporter, "deprecations", deprecations);
        emit_section(
            &*self.reporter,
            "fixes",
            fixes
                .iter()
                .map(|(k, v)| format_resolution_line(k, &v.version))
                .collect(),
        );
        emit_section(
            &*self.reporter,
            "suggested resolutions",
            resolutions
                .iter()
                .map(|(k, v)| format_resolution_line(k, &v.version))
                .collect(),
        );
        emit_section(&*self.reporter, "unresolved issues", errors);
        self.print_suggested_ignores(&yarn, ignore_suggestions)?;
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

    pub fn auto(&mut self) -> Result<(), Error> {
        let _: () = loop {
            let mut yarn = self.yarn()?;
            self.reporter
                .emit(UiEvent::Phase(crate::ui::Phase::Install));
            yarn.install()?;
            self.reporter.emit(UiEvent::Phase(crate::ui::Phase::Dedupe));
            yarn.dedupe()?;

            let mut dirty = false;
            let mut resets = vec![];
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
                    resets.push(advisory.module_name().to_owned());
                }
            }

            if !dirty || self.options.no_install() {
                break;
            }

            if !resets.is_empty() {
                yarn.locks()?.reset(&resets)?;
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

        self.check()
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

        let ids: Vec<String> = suggestions
            .keys()
            .filter(|id| !existing.contains(id.as_str()))
            .cloned()
            .collect();
        if ids.is_empty() {
            return Ok(());
        }

        let lines: Vec<String> = ids
            .iter()
            .filter_map(|id| suggestions.get(id))
            .map(|s| {
                format!(
                    "{}  {}  {}@{}",
                    s.id, s.severity, s.module_name, s.vulnerable_versions
                )
            })
            .collect();

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
                            message: format!("drop ignore {id} (within-range fix)"),
                        });
                        if yarnrc.remove_npm_audit_ignore_advisory(id) {
                            yarnrc_dirty = true;
                        }
                        resets.push(advisory.module_name().to_owned());
                    }
                }
            }
        }

        if yarnrc_dirty {
            yarnrc.save()?;
        }

        if !resets.is_empty() {
            yarn.locks()?.reset(&resets)?;
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
    severity: String,
    module_name: String,
    vulnerable_versions: String,
}

fn record_ignore_suggestion(
    map: &mut BTreeMap<String, IgnoreSuggestion>,
    advisory: &Advisory,
) {
    map.entry(advisory.id().to_owned()).or_insert_with(|| IgnoreSuggestion {
        id: advisory.id().to_owned(),
        severity: advisory.severity().to_string(),
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
