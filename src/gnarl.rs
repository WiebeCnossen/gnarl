use std::collections::{BTreeMap, HashMap, HashSet};

use nodejs_semver::{OutsideDirection, Range, Version};

use crate::{
    Error,
    audit::Advisory,
    check::Kpis,
    cmd::Options,
    npm::{Npm, Packument},
    out_fix, out_hit, out_indent, out_info,
    package::Dependency,
    parse,
    yarn::Yarn,
    yarnrc::pretty_ignore_block,
};

pub struct Gnarl {
    options: Options,
    npm: Npm,
    reset: HashSet<String>,
}

struct SuggestedFix {
    version: Version,
}

impl Gnarl {
    pub fn new(options: Options) -> Result<Self, Error> {
        Ok(Self {
            options,
            npm: Npm::new()?,
            reset: HashSet::new(),
        })
    }

    pub fn check(&mut self) -> Result<(), Error> {
        let yarn = Yarn::new(self.options.severity())?;

        let advisories = yarn.audit()?;
        let mut deprecations = vec![];
        let mut fixes = BTreeMap::new();
        let mut resolutions = BTreeMap::new();
        let mut errors = vec![];
        let mut ignore_suggestions = BTreeMap::new();
        for advisory in &advisories {
            out_hit!(
                "{}: {}@{}",
                advisory.label(),
                advisory.module_name(),
                advisory.vulnerable_versions()
            );
        }

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
            let yarn_resolutions = yarn.locks()?.for_package(advisory.module_name())?;
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

        fixes.retain(|key, _| !resolutions.contains_key(key));

        Kpis::new(
            yarn.len_dependencies(),
            yarn.len_dev_dependencies(),
            yarn.locks()?.len(),
            yarn.len_resolutions(),
            deprecations.len(),
            fixes.len() + resolutions.len() + errors.len(),
        )
        .print();

        self.print_ignore_overview(&yarn)?;

        print_section("deprecations", deprecations);
        print_section(
            "fixes",
            fixes
                .iter()
                .map(|(k, v)| format!("\"{}\": \"^{}\",", k, v.version))
                .collect(),
        );
        print_section(
            "suggested resolutions",
            resolutions
                .iter()
                .map(|(k, v)| format!("\"{}\": \"^{}\",", k, v.version))
                .collect(),
        );
        print_section("unresolved issues", errors);
        self.print_suggested_ignores(&yarn, ignore_suggestions)?;

        Ok(())
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<(), Error> {
        self.reset
            .extend(packages.iter().map(|p| p.as_ref().to_string()));
        let dirty = Yarn::new(self.options.severity())?
            .locks()?
            .reset(packages)?;

        if dirty && !self.options.no_install() {
            self.auto()?;
        }

        Ok(())
    }

    pub fn auto(&mut self) -> Result<(), Error> {
        let _: () = loop {
            let mut yarn = Yarn::new(self.options.severity())?;
            yarn.install()?;
            yarn.dedupe()?;

            let mut dirty = false;
            let mut resets = vec![];
            let mut advisories = yarn.audit()?;
            out_info!("{} advisories", advisories.len());
            let mut done = HashSet::new();

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

        let mut yarn = Yarn::new(self.options.severity())?;
        let resolutions_dirty = yarn.reset_resolutions()?;
        let ignore_resets = self.reset_ignored_advisories(&mut yarn)?;
        if (resolutions_dirty || !ignore_resets.is_empty()) && !self.options.no_install() {
            let yarn = Yarn::new(self.options.severity())?;
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

        print_section("npmAuditIgnoreAdvisories", lines);
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

        print_section("suggested ignores", lines);
        print!("{}", pretty_ignore_block(&ids));
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
                    out_fix!("drop orphan ignore {}", id);
                    if yarnrc.remove_npm_audit_ignore_advisory(id) {
                        yarnrc_dirty = true;
                    }
                }
                Some(advisory) if advisory.is_deprecation() => {}
                Some(advisory) => {
                    if self.within_range_resettable(yarn, advisory)? {
                        out_fix!("drop ignore {} (within-range fix)", id);
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
        yarn: &Yarn,
        advisory: &Advisory,
    ) -> Result<bool, Error> {
        self.npm.retrieve_packument(advisory.module_name())?;
        let packument = self.npm.packument(advisory.module_name()).cloned()?;
        let mut fixable = false;
        let mut blocked = false;
        let locks = yarn.locks()?;
        for dependent in locks.dependents(advisory.module_name())?.iter() {
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
        let locks = yarn.locks()?;
        for dependent in locks.dependents(advisory.module_name())?.iter() {
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
                out_info!(
                    "{} blocked by {}@{}",
                    advisory.root_name().unwrap_or(advisory.module_name()),
                    advisory.module_name(),
                    tree_version
                );
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
            out_info!(
                "{}@{} has no fix",
                advisory.module_name(),
                advisory.vulnerable_versions()
            );
        }

        Ok(false)
    }

    fn create_advisory(
        &mut self,
        yarn: &Yarn,
        advisory: &Advisory,
        dependent: &Dependency,
        root_name: String,
    ) -> Result<Advisory, Error> {
        self.npm.retrieve_packument(dependent.name())?;
        let packument = self.npm.packument(dependent.name())?;
        let tree_versions = yarn
            .locks()?
            .for_package(dependent.name())?
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

fn print_section(title: &str, mut lines: Vec<String>) {
    if lines.is_empty() {
        return;
    }

    out_info!("{}", title);
    lines.sort_unstable();
    lines.dedup();
    for line in lines {
        out_indent!("{}", line);
    }
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
