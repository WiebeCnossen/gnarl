## 1. Output cleanup

- [x] 1.1 Remove `# ignore:` annotations from resolution and unresolved lines in `check`
- [x] 1.2 Remove `# ignore:` from `auto` no-fix messages
- [x] 1.3 Rename the resolutions section title to `suggested resolutions`

## 2. Suggested ignores section

- [x] 2.1 Collect unique advisory IDs (with enrichment fields) from resolution and unresolved buckets during `check`
- [x] 2.2 Filter out IDs already in yarnrc `npmAuditIgnoreAdvisories`; omit the section when none remain
- [x] 2.3 Print `suggested ignores` with overview-style enriched lines, then paste-ready `npmAuditIgnoreAdvisories` YAML (new IDs only); reuse `pretty_ignore_block` if practical

## 3. Docs

- [x] 3.1 Update README Check section for `suggested ignores`, YAML paste behavior, and the `suggested resolutions` rename
