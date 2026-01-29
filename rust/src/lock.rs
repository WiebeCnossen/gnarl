use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::{collections::HashMap, fs, path::PathBuf};

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

    pub fn len(&self) -> usize {
        self.root.packages.len()
    }
}
