## Why

When gnarl saves `.yarnrc.yml` (ignore hygiene, temporary clear/restore for unfiltered audit), it always writes Unix `\n` line endings for the rewritten `npmAuditIgnoreAdvisories` block—even when the rest of the file uses Windows `\r\n`. That creates noisy diffs and fights editors/VCS that expect CRLF on Windows projects.

## What Changes

- Detect the line-ending style already used in an existing `.yarnrc.yml` when saving
- Emit the spliced/rewritten `npmAuditIgnoreAdvisories` block (and any join newlines) with that same style
- Keep unrelated file content byte-stable aside from the intentional ignore-block edit
- Add regression tests covering CRLF and LF yarnrc save paths

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `npm-audit-ignore-advisories`: Saving `.yarnrc.yml` MUST preserve the file's existing line endings instead of always writing LF

## Impact

- `src/yarnrc.rs` (`pretty_ignore_block`, splice/join helpers, `save` / `write_text`)
- Unit tests under `yarnrc` for CRLF round-trips
- No CLI, lockfile, or dependency changes
