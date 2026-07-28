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

/// Yarn-style block (LF). Prefer for stdout; file saves use [`pretty_ignore_block_with_newline`].
/// ```yaml
/// npmAuditIgnoreAdvisories:
///   - "123"
/// ```
pub(crate) fn pretty_ignore_block(ids: &[String]) -> String {
    pretty_ignore_block_with_newline(ids, "\n")
}

fn pretty_ignore_block_with_newline(ids: &[String], newline: &str) -> String {
    let mut block = format!("npmAuditIgnoreAdvisories:{newline}");
    for id in ids {
        block.push_str("  - ");
        block.push_str(&format_yaml_string(id));
        block.push_str(newline);
    }
    block
}

fn format_yaml_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// CRLF if the text contains any `\r\n`, otherwise LF.
fn detect_newline(text: &str) -> &'static str {
    if text.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}

/// Replace or remove the `npmAuditIgnoreAdvisories` block while leaving all other
/// text (comments, blank lines, key order, quoting) untouched.
fn splice_ignore_block(original: &str, ids: &[String]) -> String {
    let newline = detect_newline(original);
    let (before, after) = split_around_ignore_block(original);
    if ids.is_empty() {
        return join_preserving_newlines(&before, "", &after, newline);
    }
    join_preserving_newlines(
        &before,
        &pretty_ignore_block_with_newline(ids, newline),
        &after,
        newline,
    )
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

fn join_preserving_newlines(before: &str, middle: &str, after: &str, newline: &str) -> String {
    let mut out = String::new();
    out.push_str(before);
    if !middle.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push_str(newline);
        }
        out.push_str(middle);
    }
    if !after.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') && !after.starts_with('\n') && !after.starts_with('\r')
        {
            out.push_str(newline);
        }
        out.push_str(after);
    }
    if !out.is_empty() && !out.ends_with('\n') {
        out.push_str(newline);
    }
    out
}

fn value_to_id(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => n
            .as_u64()
            .map(|u| u.to_string())
            .or_else(|| n.as_i64().map(|i| i.to_string())),
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
        join_preserving_newlines(&before, "", &after, detect_newline(text))
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

    #[test]
    fn reads_unquoted_integer_and_mixed_ignore_ids() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        fs::write(
            &path,
            "nodeLinker: node-modules\nnpmAuditIgnoreAdvisories:\n  - 1090865\n  - \"2222222\"\n  - GHSA-xxxx-yyyy-zzzz\n  - 3333333\n",
        )
        .unwrap();

        let yarnrc = YarnRc::read(path).unwrap();
        assert_eq!(
            yarnrc.npm_audit_ignore_advisories(),
            vec![
                "1090865".to_owned(),
                "2222222".to_owned(),
                "GHSA-xxxx-yyyy-zzzz".to_owned(),
                "3333333".to_owned(),
            ]
        );
    }

    #[test]
    fn reads_flow_style_integer_ignore_ids() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        fs::write(
            &path,
            "npmAuditIgnoreAdvisories: [1090865, \"2222222\", 3333333]\n",
        )
        .unwrap();

        assert_eq!(
            YarnRc::read(path).unwrap().npm_audit_ignore_advisories(),
            vec![
                "1090865".to_owned(),
                "2222222".to_owned(),
                "3333333".to_owned(),
            ]
        );
    }

    #[test]
    fn remove_and_save_integer_ignore_rewrites_quoted_and_preserves_rest() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        let original = "# keep\nnodeLinker: node-modules\nnpmMinimalAgeGate: 1d\nnpmAuditIgnoreAdvisories:\n  - 1111111\n  - 2222222\n  - GHSA-xxxx-yyyy-zzzz\ncompressionLevel: mixed\n";
        fs::write(&path, original).unwrap();

        let before_text = text_without_ignore_block(&fs::read_to_string(&path).unwrap());
        let before = mapping_without_ignores(&read_root(&path));

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        assert!(yarnrc.remove_npm_audit_ignore_advisory("1111111"));
        yarnrc.save().unwrap();

        let after_text = fs::read_to_string(&path).unwrap();
        assert_eq!(text_without_ignore_block(&after_text), before_text);
        assert!(after_text.contains("# keep"));
        assert!(
            after_text
                .contains("npmAuditIgnoreAdvisories:\n  - \"2222222\"\n  - \"GHSA-xxxx-yyyy-zzzz\"\n")
        );
        assert_eq!(mapping_without_ignores(&read_root(&path)), before);
        assert_eq!(
            YarnRc::read(path).unwrap().npm_audit_ignore_advisories(),
            vec!["2222222".to_owned(), "GHSA-xxxx-yyyy-zzzz".to_owned()]
        );
    }

    #[test]
    fn clear_and_restore_integer_ignores_like_unfiltered_audit() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        fs::write(
            &path,
            "nodeLinker: node-modules\nnpmAuditIgnoreAdvisories:\n  - 1090865\n  - \"2222222\"\n",
        )
        .unwrap();

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        let saved = yarnrc.npm_audit_ignore_advisories();
        assert_eq!(
            saved,
            vec!["1090865".to_owned(), "2222222".to_owned()]
        );

        yarnrc.set_npm_audit_ignore_advisories(&[]);
        yarnrc.save().unwrap();
        assert!(
            YarnRc::read(path.clone())
                .unwrap()
                .npm_audit_ignore_advisories()
                .is_empty()
        );

        yarnrc.set_npm_audit_ignore_advisories(&saved);
        yarnrc.save().unwrap();
        assert_eq!(
            YarnRc::read(path).unwrap().npm_audit_ignore_advisories(),
            saved
        );
    }

    #[test]
    fn integer_ignore_id_matches_string_for_hygiene_lookups() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        fs::write(
            &path,
            "npmAuditIgnoreAdvisories:\n  - 1090865\n  - \"7777777\"\n",
        )
        .unwrap();

        let ignores = YarnRc::read(path).unwrap().npm_audit_ignore_advisories();
        let existing: std::collections::HashSet<String> = ignores.into_iter().collect();

        // Audit JSON integer ID normalized the same way as yarnrc integer entry.
        assert!(existing.contains("1090865"));
        assert!(existing.contains("7777777"));
        assert!(!existing.contains("9999999"));
    }

    #[test]
    fn value_to_id_rejects_non_integer_numbers() {
        assert_eq!(value_to_id(&Value::Number(serde_yaml::Number::from(1.5))), None);
        assert_eq!(
            value_to_id(&Value::Number(serde_yaml::Number::from(1090865u64))),
            Some("1090865".to_owned())
        );
    }

    fn is_crlf_file(text: &str) -> bool {
        text.contains("\r\n") && !text.replace("\r\n", "").contains('\n')
    }

    #[test]
    fn save_preserves_crlf_line_endings_in_ignore_block() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        let original = "# keep\r\nnodeLinker: node-modules\r\nnpmMinimalAgeGate: 1d\r\nnpmAuditIgnoreAdvisories:\r\n  - \"1111111\"\r\n  - \"2222222\"\r\n  - \"GHSA-xxxx-yyyy-zzzz\"\r\ncompressionLevel: mixed\r\n";
        assert!(is_crlf_file(original));
        fs::write(&path, original).unwrap();

        let before_text = text_without_ignore_block(&fs::read_to_string(&path).unwrap());

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        assert!(yarnrc.remove_npm_audit_ignore_advisory("1111111"));
        yarnrc.save().unwrap();

        let after_text = fs::read_to_string(&path).unwrap();
        assert!(is_crlf_file(&after_text), "save introduced LF-only newlines: {after_text:?}");
        assert_eq!(text_without_ignore_block(&after_text), before_text);
        assert!(after_text.contains(
            "npmAuditIgnoreAdvisories:\r\n  - \"2222222\"\r\n  - \"GHSA-xxxx-yyyy-zzzz\"\r\n"
        ));
        assert_eq!(
            YarnRc::read(path).unwrap().npm_audit_ignore_advisories(),
            vec!["2222222".to_owned(), "GHSA-xxxx-yyyy-zzzz".to_owned()]
        );
    }

    #[test]
    fn save_appends_ignore_block_with_crlf_when_file_uses_crlf() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".yarnrc.yml");
        let original = "# hi\r\nnodeLinker: pnp\r\nnpmMinimalAgeGate: 1d\r\n";
        fs::write(&path, original).unwrap();

        let mut yarnrc = YarnRc::read(path.clone()).unwrap();
        yarnrc.set_npm_audit_ignore_advisories(&["9999999".to_owned()]);
        yarnrc.save().unwrap();

        let after_text = fs::read_to_string(&path).unwrap();
        assert!(is_crlf_file(&after_text), "save introduced LF-only newlines: {after_text:?}");
        assert!(after_text.ends_with("npmAuditIgnoreAdvisories:\r\n  - \"9999999\"\r\n"));
    }

    #[test]
    fn detect_newline_prefers_crlf_when_present() {
        assert_eq!(detect_newline("a\nb\n"), "\n");
        assert_eq!(detect_newline("a\r\nb\r\n"), "\r\n");
        assert_eq!(detect_newline("a\nb\r\n"), "\r\n");
    }
}
