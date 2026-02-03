use std::collections::HashSet;

use nodejs_semver::Range;

use crate::{
    Error,
    audit::Advisory,
    cmd::Options,
    npm::{Npm, Packument},
    package::Dependency,
    parse,
    yarn::Yarn,
};

pub struct Gnarl {
    options: Options,
    npm: Npm,
    reset: HashSet<String>,
}

impl Gnarl {
    pub fn new(options: Options) -> Result<Self, Error> {
        let npm = Npm::new()?;
        Ok(Self {
            options,
            npm,
            reset: HashSet::new(),
        })
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<(), Error> {
        let mut yarn = Yarn::new()?;
        let dirty = yarn.reset(packages)?;

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

            while let Some(advisory) = advisories.pop() {
                dirty = self.fix(&mut yarn, advisory, &mut advisories)? || dirty;
            }

            if !dirty || self.options.no_install() {
                break;
            }
        };
        Ok(())
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

        for dependent in yarn.dependents(advisory.module_name())? {
            if has_fix(
                &packument,
                advisory.vulnerable_versions(),
                dependent.request(),
            ) {
                resets.insert(advisory.module_name().to_owned());
            } else if dependent.source() == "npm" {
                advisories.push(self.create_advisory(yarn, &advisory, &dependent)?);
            }
        }

        let mut dirty = false;
        for reset in resets.iter() {
            dirty = self.reset_one(yarn, reset)? || dirty;
        }

        if dirty {
            return Ok(true);
        }

        Ok(false)
    }

    fn reset_one(&mut self, yarn: &mut Yarn, name: &str) -> Result<bool, Error> {
        if self.reset.insert(name.to_owned()) {
            yarn.reset(&[name])?;
            Ok(true)
        } else {
            Ok(false)
        }
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
            .resolutions(dependent.name())?
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
            dependent.name().to_owned(),
            advisory.severity(),
            parse::parse_range(&vulnerable_versions)?,
            tree_versions,
            vec![],
        ))
    }
}

fn has_fix(packument: &Packument, vulnerable_versions: &Range, request: &Range) -> bool {
    packument
        .versions()
        .any(|version| request.satisfies(version) && !vulnerable_versions.satisfies(version))
}
