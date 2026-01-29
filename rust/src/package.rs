use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde_json::{Value, json};

pub struct Package {
    path: PathBuf,
    root: Value,
}

const PACKAGE_INACCESSIBLE: &'static str = "package.json is inaccessible";
const PACKAGE_CORRUPTED: &'static str = "package.json is corrupted";

impl From<std::io::Error> for crate::Error {
    fn from(_: std::io::Error) -> Self {
        PACKAGE_INACCESSIBLE.into()
    }
}

impl From<serde_json::Error> for crate::Error {
    fn from(_: serde_json::Error) -> Self {
        PACKAGE_CORRUPTED.into()
    }
}

impl Package {
    pub fn read(path: PathBuf) -> Result<Self, crate::Error> {
        let mut root: Value = serde_json::from_reader(File::open(&path)?)?;
        if root
            .get("resolutions")
            .and_then(|value| value.as_object())
            .is_none()
        {
            root["resolutions"] = json!({});
        }

        let mut result = Self { path, root };
        result.ensure_string_map("resolutions");
        result.ensure_string_map("dependencies");
        result.ensure_string_map("devDependencies");
        Ok(result)
    }

    fn ensure_string_map(&mut self, key: &str) {
        if self
            .root
            .get(key)
            .and_then(|value| value.as_object())
            .is_none()
        {
            self.root[key] = json!({});
        }
    }

    fn get_string_map(&self, key: &str) -> impl Iterator<Item = (String, String)> {
        self.root[key]
            .as_object()
            .unwrap()
            .iter()
            .map(|(key, value)| (key.to_string(), value.as_str().unwrap().to_string()))
    }

    pub fn resolutions(&self) -> HashMap<String, String> {
        self.get_string_map("resolutions").collect()
    }

    pub fn dependencies(&self) -> HashMap<String, String> {
        self.get_string_map("dependencies").collect()
    }

    pub fn dev_dependencies(&self) -> HashMap<String, String> {
        self.get_string_map("devDependencies").collect()
    }

    pub fn save(&self) -> Result<(), crate::Error> {
        // Write back with pretty printing (4 spaces indent, like npm/yarn usually do)
        let file_out = File::create(&self.path)?;
        let mut writer = BufWriter::new(file_out);
        serde_json::to_writer_pretty(&mut writer, &self.root)?;
        writer.flush()?;
        Ok(())
    }
}
