use std::{
    collections::{BTreeMap, HashMap},
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

    pub fn resolutions(&self) -> BTreeMap<String, String> {
        self.get_string_map("resolutions").collect()
    }

    pub fn set_resolution(&mut self, package: &str, request: impl Into<String>) {
        self.root["resolutions"][package] = json!(request.into());
    }

    /// Persist suggested pins. Empty snapshot does not write. Values are `^{version}`.
    pub fn apply_suggested_resolutions(
        &mut self,
        entries: &[(String, String)],
    ) -> Result<bool, crate::Error> {
        if entries.is_empty() {
            return Ok(false);
        }
        for (key, version) in entries {
            self.set_resolution(key, format!("^{version}"));
        }
        self.save()?;
        Ok(true)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_pkg(dir: &std::path::Path, json: &str) -> PathBuf {
        let path = dir.join("package.json");
        fs::write(&path, json).unwrap();
        path
    }

    #[test]
    fn apply_suggested_resolutions_inserts_caret_and_keeps_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_pkg(
            dir.path(),
            r#"{"resolutions":{"keep@^1":"^1.0.0"}}"#,
        );
        let mut project = Project::read(path.clone()).unwrap();
        let wrote = project
            .apply_suggested_resolutions(&[("pkg@^1".into(), "1.2.3".into())])
            .unwrap();
        assert!(wrote);
        let again = Project::read(path).unwrap();
        let res = again.resolutions();
        assert_eq!(res.get("keep@^1").map(String::as_str), Some("^1.0.0"));
        assert_eq!(res.get("pkg@^1").map(String::as_str), Some("^1.2.3"));
    }

    #[test]
    fn empty_snapshot_does_not_write() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_pkg(dir.path(), r#"{"name":"x"}"#);
        let before = fs::read_to_string(&path).unwrap();
        let mut project = Project::read(path.clone()).unwrap();
        assert!(!project.apply_suggested_resolutions(&[]).unwrap());
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
    }
}
