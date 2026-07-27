use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

use serde_yaml::Value;

use crate::Error;

const NPM_AUDIT_IGNORE_ADVISORIES: &str = "npmAuditIgnoreAdvisories";

pub struct YarnRc {
    path: PathBuf,
    root: Value,
}

impl YarnRc {
    pub fn read(path: PathBuf) -> Result<Self, Error> {
        if !path.exists() {
            return Ok(Self {
                path,
                root: Value::Mapping(serde_yaml::Mapping::new()),
            });
        }

        let root: Value = serde_yaml::from_reader(File::open(&path)?)?;
        Ok(Self { path, root })
    }

    pub fn npm_audit_ignore_advisories(&self) -> Vec<String> {
        match self.root.get(NPM_AUDIT_IGNORE_ADVISORIES) {
            Some(Value::Sequence(seq)) => seq.iter().filter_map(value_to_id).collect(),
            _ => Vec::new(),
        }
    }

    pub fn set_npm_audit_ignore_advisories(&mut self, ids: &[String]) {
        let seq = ids
            .iter()
            .map(|id| Value::String(id.clone()))
            .collect::<Vec<_>>();
        match &mut self.root {
            Value::Mapping(map) => {
                if seq.is_empty() {
                    map.remove(Value::String(NPM_AUDIT_IGNORE_ADVISORIES.to_owned()));
                } else {
                    map.insert(
                        Value::String(NPM_AUDIT_IGNORE_ADVISORIES.to_owned()),
                        Value::Sequence(seq),
                    );
                }
            }
            _ => {
                let mut map = serde_yaml::Mapping::new();
                if !seq.is_empty() {
                    map.insert(
                        Value::String(NPM_AUDIT_IGNORE_ADVISORIES.to_owned()),
                        Value::Sequence(seq),
                    );
                }
                self.root = Value::Mapping(map);
            }
        }
    }

    pub fn remove_npm_audit_ignore_advisory(&mut self, id: &str) -> bool {
        let mut ids = self.npm_audit_ignore_advisories();
        let before = ids.len();
        ids.retain(|existing| existing != id);
        if ids.len() == before {
            return false;
        }
        self.set_npm_audit_ignore_advisories(&ids);
        true
    }

    pub fn save(&self) -> Result<(), Error> {
        let ids = self.npm_audit_ignore_advisories();

        if !self.path.exists() {
            if ids.is_empty() {
                return Ok(());
            }
            return write_text(&self.path, &pretty_ignore_block(&ids));
        }

        let original = fs::read_to_string(&self.path)?;
        let updated = splice_ignore_block(&original, &ids);
        write_text(&self.path, &updated)
    }
}

fn write_text(path: &PathBuf, text: &str) -> Result<(), Error> {
    let file_out = File::create(path)?;
    let mut writer = BufWriter::new(file_out);
    writer.write_all(text.as_bytes())?;
    writer.flush()?;
    Ok(())
}

/// Yarn-style block:
/// ```yaml
/// npmAuditIgnoreAdvisories:
///   - "123"
/// ```
pub(crate) fn pretty_ignore_block(ids: &[String]) -> String {
    let mut block = String::from("npmAuditIgnoreAdvisories:\n");
    for id in ids {
        block.push_str("  - ");
        block.push_str(&format_yaml_string(id));
        block.push('\n');
    }
    block
}

fn format_yaml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Replace or remove the `npmAuditIgnoreAdvisories` block while leaving all other
/// text (comments, blank lines, key order, quoting) untouched.
fn splice_ignore_block(original: &str, ids: &[String]) -> String {
    let (before, after) = split_around_ignore_block(original);
    if ids.is_empty() {
        return join_preserving_newlines(&before, "", &after);
    }
    join_preserving_newlines(&before, &pretty_ignore_block(ids), &after)
}

fn split_around_ignore_block(original: &str) -> (String, String) {
    let lines: Vec<&str> = original.split_inclusive('\n').collect();
    let mut start = None;
    let mut end = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = trim_yaml_line(line);
        if start.is_none() {
            if trimmed == NPM_AUDIT_IGNORE_ADVISORIES
                || trimmed.starts_with(&format!("{NPM_AUDIT_IGNORE_ADVISORIES}:"))
            {
                start = Some(idx);
                let after_key = trimmed
                    .strip_prefix(NPM_AUDIT_IGNORE_ADVISORIES)
                    .and_then(|rest| rest.strip_prefix(':'))
                    .map(str::trim)
                    .unwrap_or("");
                // Inline form: `npmAuditIgnoreAdvisories: []` or `npmAuditIgnoreAdvisories: ["a"]`
                if !after_key.is_empty() {
                    end = Some(idx + 1);
                    break;
                }
                end = Some(idx + 1);
            }
            continue;
        }

        let end_idx = end.unwrap_or(idx);
        let trimmed = trim_yaml_line(line);
        if trimmed.is_empty() || is_yaml_list_item(trimmed) {
            end = Some(idx + 1);
            continue;
        }
        // Hit the next top-level key (or other content).
        let _ = end_idx;
        break;
    }

    match (start, end) {
        (Some(s), Some(e)) => {
            let before = lines[..s].concat();
            let after = lines[e..].concat();
            (before, after)
        }
        _ => (original.to_owned(), String::new()),
    }
}

fn trim_yaml_line(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n']).trim()
}

fn is_yaml_list_item(trimmed: &str) -> bool {
    trimmed.starts_with("- ") || trimmed == "-"
}

fn join_preserving_newlines(before: &str, middle: &str, after: &str) -> String {
    let mut out = String::new();
    out.push_str(before);
    if !middle.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(middle);
    }
    if !after.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') && !after.starts_with('\n') {
            out.push('\n');
        }
        out.push_str(after);
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn value_to_id(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const SAMPLE_YARNRC: &str = r#"# keep this comment
nodeLinker: node-modules

npmMinimalAgeGate: 1d
yarnPath: .yarn/releases/yarn-4.15.0.cjs
enableGlobalCache: false
packageExtensions:
  "webpack@*":
    peerDependencies:
      webpack-cli: "*"
npmAuditIgnoreAdvisories:
  - "1111111"
  - "2222222"
  - GHSA-xxxx-yyyy-zzzz
compressionLevel: mixed
"#;

    fn write_sample(path: &std::path::Path) {
        fs::write(path, SAMPLE_YARNRC).unwrap();
    }

    fn mapping_without_ignores(root: &Value) -> serde_yaml::Mapping {
        let mut map = root
            .as_mapping()
            .expect("yarnrc root must be a mapping")
            .clone();
        map.remove(Value::String(NPM_AUDIT_IGNORE_ADVISORIES.to_owned()));
        map
    }

    fn read_root(path: &std::path::Path) -> Value {
        serde_yaml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
    }

    fn text_without_ignore_block(text: &str) -> String {
        let (before, after) = split_around_ignore_block(text);
        join_preserving_newlines(&before, "", &after)
    }

    #[test]
    fn remove_ignore_preserves_other_yaml_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        write_sample(&path);

        let before = mapping_without_ignores(&read_root(&path));
        let before_text = text_without_ignore_block(&fs::read_to_string(&path).unwrap());

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        assert!(yarnrc.remove_npm_audit_ignore_advisory("1111111"));
        yarnrc.save().unwrap();

        let after_text = fs::read_to_string(&path).unwrap();
        assert_eq!(text_without_ignore_block(&after_text), before_text);
        assert!(after_text.contains("# keep this comment"));
        assert!(after_text.contains("npmAuditIgnoreAdvisories:\n  - \"2222222\"\n  - \"GHSA-xxxx-yyyy-zzzz\"\n"));

        let after_root = read_root(&path);
        assert_eq!(mapping_without_ignores(&after_root), before);
        assert_eq!(
            YarnRc::read(path).unwrap().npm_audit_ignore_advisories(),
            vec!["2222222".to_owned(), "GHSA-xxxx-yyyy-zzzz".to_owned(),]
        );
    }

    #[test]
    fn clearing_all_ignores_preserves_other_yaml_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        write_sample(&path);

        let before = mapping_without_ignores(&read_root(&path));
        let before_text = text_without_ignore_block(&fs::read_to_string(&path).unwrap());

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        yarnrc.set_npm_audit_ignore_advisories(&[]);
        yarnrc.save().unwrap();

        let after_text = fs::read_to_string(&path).unwrap();
        assert_eq!(text_without_ignore_block(&after_text), before_text);
        assert!(after_text.contains("# keep this comment"));
        assert!(!after_text.contains("npmAuditIgnoreAdvisories"));

        let after_root = read_root(&path);
        assert_eq!(mapping_without_ignores(&after_root), before);
        assert!(
            YarnRc::read(path)
                .unwrap()
                .npm_audit_ignore_advisories()
                .is_empty()
        );
    }

    #[test]
    fn set_ignores_on_file_without_key_preserves_other_yaml_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        let original = "# hi\nnodeLinker: pnp\nnpmMinimalAgeGate: 1d\nyarnPath: .yarn/releases/yarn-4.15.0.cjs\n";
        fs::write(&path, original).unwrap();

        let before = mapping_without_ignores(&read_root(&path));

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        yarnrc.set_npm_audit_ignore_advisories(&["9999999".to_owned()]);
        yarnrc.save().unwrap();

        let after_text = fs::read_to_string(&path).unwrap();
        assert!(after_text.starts_with("# hi\nnodeLinker: pnp\n"));
        assert!(after_text.ends_with("npmAuditIgnoreAdvisories:\n  - \"9999999\"\n"));

        let after_root = read_root(&path);
        assert_eq!(mapping_without_ignores(&after_root), before);
        assert_eq!(
            YarnRc::read(path).unwrap().npm_audit_ignore_advisories(),
            vec!["9999999".to_owned()]
        );
    }

    #[test]
    fn missing_yarnrc_save_without_ignores_does_not_create_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");

        let yarnrc = YarnRc::read(path.clone()).unwrap();
        yarnrc.save().unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn pretty_ignore_block_uses_two_space_indent_and_double_quotes() {
        assert_eq!(
            pretty_ignore_block(&["111".to_owned(), "GHSA-ab".to_owned()]),
            "npmAuditIgnoreAdvisories:\n  - \"111\"\n  - \"GHSA-ab\"\n"
        );
    }
}
