use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde_json::{Value, json};

pub struct Project {
    path: PathBuf,
    root: Value,
}

impl Project {
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

    pub fn dependencies(&self) -> HashMap<String, String> {
        self.get_string_map("dependencies").collect()
    }

    pub fn dev_dependencies(&self) -> HashMap<String, String> {
        self.get_string_map("devDependencies").collect()
    }

    pub fn resolutions(&self) -> HashMap<String, String> {
        self.get_string_map("resolutions").collect()
    }

    pub fn set_resolution(&mut self, package: &str, request: impl Into<String>) {
        self.root["resolutions"][package] = json!(request.into());
    }

    pub fn reset_resolution(&mut self, package: &str) {
        self.root["resolutions"]
            .as_object_mut()
            .unwrap()
            .remove(package);
    }

    pub fn save(&self) -> Result<(), crate::Error> {
        // Write back with pretty printing (4 spaces indent, like npm/yarn usually do)
        let file_out = File::create(&self.path)?;
        let mut writer = BufWriter::new(file_out);
        serde_json::to_writer_pretty(&mut writer, &self.root)?;
        writeln!(&mut writer)?;
        writer.flush()?;
        Ok(())
    }
}
