## Why

gnarl already classifies advisories into within-range fixes, outside-range resolutions, and unresolved issues, but it never surfaces Yarn's audit IDs or maintains `.yarnrc.yml` `npmAuditIgnoreAdvisories`. Teams use that list as a third remediation path (silence an advisory), yet gnarl neither suggests IDs nor cleans entries that are orphaned or superseded by a within-range update.

## What Changes

- When `check` suggests a resolution or reports no fix / unresolved issue, also print the Yarn `npm audit` advisory ID that could be added to `npmAuditIgnoreAdvisories`
- Print an overview of current `npmAuditIgnoreAdvisories` entries enriched with package name and severity (from an unfiltered audit)
- In `auto`, drop ignore entries that are orphans (ID absent from unfiltered audit) or that have a within-range fix available; when dropping for a within-range fix, also reset that package so the update can apply
- After ignore-list mutations that require a tree refresh, run `install` + `dedupe` when installs are enabled (same pattern as resolution cleanup)
- Read/write `.yarnrc.yml` for `npmAuditIgnoreAdvisories` (using existing `serde_yaml`)

## Capabilities

### New Capabilities

- `npm-audit-ignore-advisories`: Suggest audit IDs alongside resolution/unresolved output, overview current ignores with package and severity, and auto-drop orphan or within-range-fixable ignore entries

### Modified Capabilities

<!-- none - parallel to auto-resolutions-cleanup; no requirement change to that capability -->

## Impact

- New `.yarnrc.yml` reader/writer (likely alongside `Project`)
- `src/gnarl.rs` (`check`, `auto`): ignore overview, ID hints, ignore hygiene + optional reset/install
- `src/yarn.rs`: unfiltered audit path (bypass configured ignore list)
- `src/audit.rs`: expose ID in user-facing output paths
- `README.md`: document ignore overview and auto-drop behavior
- CLI surface unchanged (behavior of `check` / default `auto` only)
