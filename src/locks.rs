use crate::Error;
use nodejs_semver::Range;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
};

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
    entries: Vec<Lock>,
    by_name: HashMap<String, Vec<usize>>,
    dependents_of: HashMap<String, Vec<Dependency>>,
}

impl Locks {
    pub fn read(path: PathBuf) -> Result<Self, Error> {
        let content = fs::read_to_string(&path)?;
        let root: YarnLockV2 = serde_yaml::from_str(&content)?;
        let mut locks = Self {
            path,
            root,
            entries: Vec::new(),
            by_name: HashMap::new(),
            dependents_of: HashMap::new(),
        };
        locks.rebuild_indexes()?;
        Ok(locks)
    }

    fn rebuild_indexes(&mut self) -> Result<(), Error> {
        self.entries = materialize_entries(&self.root)?;
        self.by_name.clear();
        self.dependents_of.clear();

        for (idx, lock) in self.entries.iter().enumerate() {
            self.by_name
                .entry(lock.package().name().to_owned())
                .or_default()
                .push(idx);

            for dependency in lock.dependencies() {
                self.dependents_of
                    .entry(dependency.name().to_owned())
                    .or_default()
                    .push(Dependency::new(
                        lock.package().name().to_owned(),
                        lock.package().source().to_owned(),
                        dependency.request().clone(),
                    ));
            }
        }

        Ok(())
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

    pub fn for_package(&self, package: &str) -> Vec<&Lock> {
        self.by_name
            .get(package)
            .map(|indexes| indexes.iter().map(|&idx| &self.entries[idx]).collect())
            .unwrap_or_default()
    }

    pub fn all(&self) -> &[Lock] {
        &self.entries
    }

    pub fn dependents(&self, package_name: &str) -> &[Dependency] {
        self.dependents_of
            .get(package_name)
            .map(Vec::as_slice)
            .unwrap_or(&[])
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
        self.rebuild_indexes()?;
        Ok(true)
    }
}

fn materialize_entries(root: &YarnLockV2) -> Result<Vec<Lock>, Error> {
    root.packages
        .iter()
        .map(|(key, value)| materialize_entry(key, value))
        .collect()
}

fn materialize_entry(key: &str, value: &Value) -> Result<Lock, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SAMPLE_LOCK: &str = r#"__metadata:
  version: 6
  cacheKey: 8

"left-pad@npm:^1.0.0":
  version: 1.3.0
  resolution: "left-pad@npm:1.3.0"
  languageName: node
  linkType: hard

"parent@npm:^1.0.0":
  version: 1.0.0
  resolution: "parent@npm:1.0.0"
  dependencies:
    left-pad: "npm:^1.0.0"
  languageName: node
  linkType: hard

"other@npm:^2.0.0":
  version: 2.0.0
  resolution: "other@npm:2.0.0"
  dependencies:
    left-pad: "npm:^1.2.0"
  languageName: node
  linkType: hard
"#;

    fn write_sample(dir: &std::path::Path) -> PathBuf {
        let path = dir.join("yarn.lock");
        fs::write(&path, SAMPLE_LOCK).unwrap();
        path
    }

    #[test]
    fn for_package_uses_index() {
        let dir = tempfile::tempdir().unwrap();
        let locks = Locks::read(write_sample(dir.path())).unwrap();

        let left_pad = locks.for_package("left-pad");
        assert_eq!(left_pad.len(), 1);
        assert_eq!(
            left_pad[0].package().version().unwrap().to_string(),
            "1.3.0"
        );
        assert!(locks.for_package("missing").is_empty());
        assert_eq!(locks.all().len(), 3);
    }

    #[test]
    fn dependents_uses_reverse_index() {
        let dir = tempfile::tempdir().unwrap();
        let locks = Locks::read(write_sample(dir.path())).unwrap();

        let dependents = locks.dependents("left-pad");
        assert_eq!(dependents.len(), 2);
        let mut names: Vec<_> = dependents.iter().map(Dependency::name).collect();
        names.sort_unstable();
        assert_eq!(names, vec!["other", "parent"]);
        assert!(locks.dependents("missing").is_empty());
    }

    #[test]
    fn reset_rebuilds_indexes() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_sample(dir.path());
        let mut locks = Locks::read(path).unwrap();

        assert!(locks.reset(&["left-pad"]).unwrap());
        assert!(locks.for_package("left-pad").is_empty());
        // Parent entries still declare left-pad as a dependency.
        assert_eq!(locks.dependents("left-pad").len(), 2);
        assert_eq!(locks.all().len(), 2);
        assert_eq!(locks.for_package("parent").len(), 1);
    }
}
