## Why

Yarn accepts `npmAuditIgnoreAdvisories` entries as either YAML strings or unquoted integers (both are common in real `.yarnrc.yml` files). gnarl currently crashes or otherwise fails when those IDs are integers, so projects that Yarn treats as valid cannot run `check` / `auto`.

## What Changes

- Accept advisory IDs in `.yarnrc.yml` `npmAuditIgnoreAdvisories` whether each list item is a YAML string or an integer scalar
- Normalize both forms to the same string representation used for overview, suggested-ignores filtering, remove, and auto-drop
- Keep writing ignore IDs in the existing quoted-string form when gnarl saves yarnrc (no change to output style)
- Add regression coverage so integer-form yarnrc entries no longer break read/remove/save or ignore hygiene

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `npm-audit-ignore-advisories`: Require that ignore-list entries be accepted as either string or integer YAML scalars, normalized for all ignore overview / suggest / auto-drop behavior

## Impact

- `src/yarnrc.rs` (parse/normalize of `npmAuditIgnoreAdvisories` list items; tests)
- Possibly `src/yarn.rs` / `src/gnarl.rs` if any call site assumes string-only values
- Audit JSON `ID` already accepts int-or-string; confirm ignore-list comparison stays consistent with that normalization
