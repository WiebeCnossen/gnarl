use std::{collections::HashMap, fs, path::PathBuf};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;

#[derive(Debug, Deserialize, Serialize)]
struct YarnLockV2 {
    #[serde(rename = "__metadata")]
    metadata: Option<Value>,  // or your struct if you know the shape
    // The rest is package@spec → mapping of version, resolution, etc.
    #[serde(flatten)]
    packages: HashMap<String, Value>,
}

pub struct Lock {
    path: PathBuf,
    root: YarnLockV2,
}

const LOCK_CORRUPTED: &'static str = "yarn.lock is corrupted";

impl From<serde_yaml::Error> for crate::Error {
    fn from(_: serde_yaml::Error) -> Self {
        LOCK_CORRUPTED.into()
    }
}

impl Lock {
    pub fn read(path: PathBuf) -> Result<Self, crate::Error> {
        let content = fs::read_to_string(&path)?;
        let root: YarnLockV2 = serde_yaml::from_str(&content)?;
        Ok(Self { path, root })
    }

    pub fn reset(&mut self, package: &str) {
        self.root.packages.retain(|k, _| !k.starts_with(package));
    }

    pub fn save(&self) -> Result<(), crate::Error> {
        let content = serde_yaml::to_string(&self.root)?;
        fs::write(&self.path, content)?;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.root.packages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.packages.is_empty()
    }
}