use crate::Error;
use nodejs_semver::Range;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{Package, out_fix, package::Dependency, parse};

#[derive(Debug, Deserialize, Serialize)]
struct YarnLockV2 {
    #[serde(rename = "__metadata")]
    metadata: Option<Value>, // or your struct if you know the shape
    // The rest is package@spec → mapping of version, resolution, etc.
    #[serde(flatten)]
    packages: BTreeMap<String, Value>,
}

pub struct Locks {
    path: PathBuf,
    root: YarnLockV2,
}

impl Locks {
    pub fn read(path: PathBuf) -> Result<Self, Error> {
        let content = fs::read_to_string(&path)?;
        let root: YarnLockV2 = serde_yaml::from_str(&content)?;

        Ok(Self { path, root })
    }

    fn reset_one(&mut self, package: &str) -> bool {
        let len = self.root.packages.len();
        self.root.packages.retain(|k, _| {
            if let Some(tail) = k.strip_prefix(package)
                && tail.starts_with('@')
            {
                false
            } else {
                true
            }
        });

        if len == self.root.packages.len() {
            return false;
        }

        out_fix!("reset {}", package);
        true
    }

    fn save(&self) -> Result<(), Error> {
        let content = serde_yaml::to_string(&self.root)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn for_package(&self, package: &str) -> Result<Vec<Lock>, Error> {
        Ok(self
            .all()?
            .into_iter()
            .filter(|lock| lock.package().name() == package)
            .collect())
    }

    pub fn all(&self) -> Result<Vec<Lock>, Error> {
        self.root
            .packages
            .iter()
            .map(|(key, value)| {
                let first = parse::split_first(key, ", ");
                let dependency = Dependency::parse(first);
                let (name, source) = if let Ok(dependency) = &dependency {
                    (dependency.name(), dependency.source())
                } else {
                    let (name, qualified_version) = parse::split_name(first)?;
                    let (source, _) = parse::split_qualified(qualified_version)?;
                    (name, source)
                };

                let version = value
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0.0.0");
                let package = Package::new(
                    name.to_owned(),
                    source.to_owned(),
                    Some(parse::parse_version(version)?),
                );

                let requests = key
                    .split(',')
                    .flat_map(|qualified| {
                        let (_, range) = parse::split_qualified(qualified).ok()?;
                        parse::parse_range(range).ok()
                    })
                    .collect();

                let original_requests = key
                    .split(',')
                    .flat_map(|qualified| {
                        let (_, range) = parse::split_qualified(qualified).ok()?;
                        Some(range.to_owned())
                    })
                    .collect();

                let dependencies =
                    if let Some(mapping) = value.get("dependencies").and_then(|v| v.as_mapping()) {
                        mapping
                            .iter()
                            .map(|(dependency, range)| {
                                Dependency::from_qualified_range(
                                    dependency.as_str().unwrap(),
                                    range.as_str().unwrap(),
                                )
                            })
                            .collect::<Result<Vec<Dependency>, Error>>()?
                    } else {
                        Vec::new()
                    };

                Ok(Lock::new(
                    package,
                    requests,
                    original_requests,
                    dependencies,
                ))
            })
            .collect::<Result<Vec<Lock>, Error>>()
    }

    pub fn dependents(&self, package_name: &str) -> Result<Vec<Dependency>, Error> {
        let resolutions = self.all()?;
        let dependents = resolutions
            .into_iter()
            .flat_map(|resolution| {
                resolution
                    .dependencies()
                    .iter()
                    .filter(|dependency| dependency.name() == package_name)
                    .map(|dependency| {
                        Dependency::new(
                            resolution.package().name().to_owned(),
                            resolution.package().source().to_owned(),
                            dependency.request().clone(),
                        )
                    })
                    .collect::<Vec<Dependency>>()
            })
            .collect();
        Ok(dependents)
    }

    pub fn len(&self) -> usize {
        self.root.packages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.packages.is_empty()
    }

    pub fn reset(&mut self, packages: &[impl AsRef<str>]) -> Result<bool, Error> {
        let mut dirty = false;
        for package in packages {
            dirty = self.reset_one(package.as_ref()) || dirty;
        }

        if !dirty {
            return Ok(false);
        }

        self.save()?;
        Ok(true)
    }
}

pub struct Lock {
    package: Package,
    requests: Vec<Range>,
    original_requests: Vec<String>,
    dependencies: Vec<Dependency>,
}

impl Lock {
    pub fn new(
        package: Package,
        requests: Vec<Range>,
        original_requests: Vec<String>,
        dependencies: Vec<Dependency>,
    ) -> Self {
        Self {
            package,
            requests,
            original_requests,
            dependencies,
        }
    }

    pub fn package(&self) -> &Package {
        &self.package
    }

    pub fn requests(&self) -> &[Range] {
        &self.requests
    }

    pub fn original(&self, request: &Range) -> Result<&str, Error> {
        self.original_requests
            .iter()
            .find(|original_request| {
                if let Ok(parsed) = parse::parse_range(original_request)
                    && parsed.eq(request)
                {
                    true
                } else {
                    false
                }
            })
            .map(|original_request| original_request.as_str())
            .ok_or_else(|| format!("Original request for {} not found", request).into())
    }

    pub fn dependencies(&self) -> &[Dependency] {
        &self.dependencies
    }
}
