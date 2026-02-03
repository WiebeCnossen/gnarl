use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    Package,
    package::{Dependency, Resolution},
    parse,
};

#[derive(Debug, Deserialize, Serialize)]
struct YarnLockV2 {
    #[serde(rename = "__metadata")]
    metadata: Option<Value>, // or your struct if you know the shape
    // The rest is package@spec → mapping of version, resolution, etc.
    #[serde(flatten)]
    packages: HashMap<String, Value>,
}

pub struct Lock {
    path: PathBuf,
    root: YarnLockV2,
}

impl Lock {
    pub fn read(path: PathBuf) -> Result<Self, crate::Error> {
        let content = fs::read_to_string(&path)?;
        let root: YarnLockV2 = serde_yaml::from_str(&content)?;

        Ok(Self { path, root })
    }

    pub fn reset(&mut self, package: &str) -> bool {
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

        println!("Reset {}", package);
        true
    }

    pub fn save(&self) -> Result<(), crate::Error> {
        let content = serde_yaml::to_string(&self.root)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn resolutions(&self, package: &str) -> Result<Vec<Resolution>, crate::Error> {
        Ok(self
            .all_resolutions()?
            .into_iter()
            .filter(|resolution| resolution.package().name() == package)
            .collect())
    }

    fn all_resolutions(&self) -> Result<Vec<Resolution>, crate::Error> {
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
                            .collect::<Result<Vec<Dependency>, crate::Error>>()?
                    } else {
                        Vec::new()
                    };

                Ok(Resolution::new(package, requests, dependencies))
            })
            .collect::<Result<Vec<Resolution>, crate::Error>>()
    }

    pub fn dependents(&self, package_name: &str) -> Result<Vec<Dependency>, crate::Error> {
        let resolutions = self.all_resolutions()?;
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
}
