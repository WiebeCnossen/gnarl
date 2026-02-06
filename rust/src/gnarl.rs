use std::collections::{BTreeMap, HashSet};

use nodejs_semver::{OutsideDirection, Range, Version};

use crate::{
    Error,
    audit::Advisory,
    check::Kpis,
    cmd::Options,
    npm::{Npm, Packument},
    out_indent, out_info,
    package::Dependency,
    parse,
    yarn::Yarn,
};

pub struct Gnarl {
    options: Options,
    npm: Npm,
}

impl Gnarl {
    pub fn new(options: Options) -> Result<Self, Error> {
        let npm = Npm::new()?;
        Ok(Self { options, npm })
    }

    pub fn check(&mut self) -> Result<(), Error> {
        let yarn = Yarn::new()?;

        let advisories = yarn.audit()?;
        let mut deprecations = vec![];
        let mut fixes = BTreeMap::new();
        let mut resolutions = BTreeMap::new();
        let mut errors = vec![];
        for advisory in advisories {
            if advisory.id().contains(" (deprecation)") {
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
                        add_fix(&mut fixes, advisory.module_name(), original_request, fix);
                    } else if let Some(fix) =
                        get_resolution(packument, advisory.vulnerable_versions(), request)
                    {
                        add_fix(
                            &mut resolutions,
                            advisory.module_name(),
                            original_request,
                            fix,
                        );
                    } else {
                        errors.push(format!(
                            "\"{}@{}\"",
                            advisory.module_name(),
                            original_request
                        ));
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

        print_section("deprecations", deprecations);
        print_section(
            "fixes",
            fixes
                .iter()
                .map(|(k, v)| format!("\"{}\": \"^{}\",", k, v))
                .collect(),
        );
        print_section(
            "resolutions",
            resolutions
                .iter()
                .map(|(k, v)| format!("\"{}\": \"^{}\",", k, v))
                .collect(),
        );
        print_section("unresolved issues", errors);

        Ok(())
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<(), Error> {
        let dirty = Yarn::new()?.locks()?.reset(packages)?;

        if dirty && !self.options.no_install() {
            self.auto()?;
        }

        Ok(())
    }

    pub fn auto(&mut self) -> Result<(), Error> {
        let _: () = loop {
            let mut yarn = Yarn::new()?;
            yarn.install()?;
            yarn.dedupe()?;

            let mut dirty = false;
            let mut advisories = yarn.audit()?;
            out_info!("{} advisories", advisories.len());
            let mut done = HashSet::new();

            while let Some(advisory) = advisories.pop() {
                if done.insert(format!("{} {}", advisory.id(), advisory.module_name())) {
                    dirty = self.fix(&mut yarn, advisory, &mut advisories)? || dirty;
                }
            }

            if !dirty || self.options.no_install() {
                break;
            }
        };

        Yarn::new()?.reset_resolutions()?;
        self.check()
    }

    fn fix(
        &mut self,
        yarn: &mut Yarn,
        advisory: Advisory,
        advisories: &mut Vec<Advisory>,
    ) -> Result<bool, Error> {
        self.npm.retrieve_packument(advisory.module_name())?;
        let packument = self.npm.packument(advisory.module_name()).cloned()?;
        let mut resets = HashSet::new();
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
                continue;
            }

            blocked = true;
            if dependent.source() == "npm" {
                advisories.push(self.create_advisory(yarn, &advisory, dependent)?);
            }
        }

        if fixable && !blocked {
            resets.insert(advisory.module_name().to_owned());
        }

        if yarn.locks()?.reset(&resets.iter().collect::<Vec<_>>())? {
            return Ok(true);
        }

        Ok(false)
    }

    fn create_advisory(
        &mut self,
        yarn: &Yarn,
        advisory: &Advisory,
        dependent: &Dependency,
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

fn add_fix(
    map: &mut BTreeMap<String, Version>,
    package: &str,
    request: &str,
    resolution: &Version,
) {
    map.entry(format!("{}@{}", package, request))
        .and_modify(|v| {
            if (*v).lt(resolution) {
                *v = resolution.to_owned();
            }
        })
        .or_insert(resolution.to_owned());
}
