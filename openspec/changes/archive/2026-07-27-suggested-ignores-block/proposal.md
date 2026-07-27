## Why

Inline `# ignore: <id>` annotations on resolution and unresolved lines make those sections hard to copy-paste into `package.json` (or elsewhere). Teams want ignore candidates as a dedicated, paste-ready `.yarnrc.yml` fragment instead.

## What Changes

- Remove all inline `# ignore: <id>` annotations from `check` resolution/unresolved output and from `auto` no-fix messages
- Rename the `resolutions` section title to `suggested resolutions`
- Add a `suggested ignores` section: enriched lines (same form as the current `npmAuditIgnoreAdvisories` overview), followed by a YAML block labeled `npmAuditIgnoreAdvisories` that can be pasted into `.yarnrc.yml`
- Suggested IDs come from outside-range resolution candidates and unresolved / no-fix issues; exclude deprecations, within-range fixes, and IDs already listed in yarnrc
- YAML lists new suggestions only (not a merge with existing ignores); omit the whole section when there is nothing to suggest
- This supersedes the earlier “mention audit ID on the same line” check/auto output requirements

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `npm-audit-ignore-advisories`: Replace inline ID hints with a separate `suggested ignores` block (enriched overview + copy-paste YAML); rename resolutions section; drop inline `# ignore:` everywhere

## Impact

- `src/gnarl.rs` (`check` section titles/formatting, suggested-ignores printer; `auto` no-fix message)
- Possibly reuse `pretty_ignore_block` from `src/yarnrc.rs` for the YAML fragment
- `README.md`: document suggested ignores + section rename
- No CLI flag or yarnrc write-path changes (suggestions remain print-only)
