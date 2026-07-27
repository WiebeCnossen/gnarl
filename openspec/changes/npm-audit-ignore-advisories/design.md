## Context

gnarl's `check` and `auto` flows consume `yarn npm audit --json` advisories, classify them (within-range fix, outside-range resolution, unresolved), and for auto within-range cases reset packages and loop install/dedupe. Advisory IDs are already parsed on `Advisory` but barely shown. `.yarnrc.yml` is not read today, though `serde_yaml` is already a dependency and a TODO mentions reading `npmMinimalAgeGate`.

Yarn's `npmAuditIgnoreAdvisories` filters matching IDs out of normal audit output, so hygiene and enrichment require an unfiltered audit. Numeric audit IDs can rotate (yarnpkg/berry#6438); orphan auto-drop handles vanished IDs explicitly.

## Goals / Non-Goals

**Goals:**

- Surface audit IDs next to resolution suggestions and no-fix / unresolved output
- Print an overview of current `npmAuditIgnoreAdvisories` with ID, package, and severity when known
- Auto-drop ignore entries that are orphans or superseded by a within-range fix; reset packages in the latter case
- Persist yarnrc changes; refresh the lockfile when drops imply package resets

**Non-Goals:**

- Auto-adding new IDs to `npmAuditIgnoreAdvisories` (suggest only)
- Translating IDs to CVE/GHSA strings beyond what Yarn reports
- Ignoring by package name (`npmAuditExcludePackages`)
- Reading/writing `npmMinimalAgeGate` (separate TODO)
- Changing severity filtering or audit CLI flags beyond ignore bypass
- Re-entering a full advisory fix loop solely because orphan IDs were dropped

## Decisions

1. **Unfiltered audit via temporary yarnrc clear (not env)**
   Prefer: briefly clear `npmAuditIgnoreAdvisories` in `.yarnrc.yml`, run audit, then restore the previous list (even on audit failure). Yarn documents that env overrides do not support arrays/objects, so `YARN_NPM_AUDIT_IGNORE_ADVISORIES` is not viable.
   Alternative considered: env override — rejected given Yarn's documented limitation.
   Risk accepted: process kill mid-audit could leave ignores cleared; restore is best-effort on the happy/error paths we control.

2. **Yarnrc module parallel to `Project`**
   Prefer: small dedicated type (e.g. `YarnRc`) that reads/writes `.yarnrc.yml`, exposes `npm_audit_ignore_advisories() -> Vec<String>`, and can remove IDs and save while preserving unrelated keys/formatting as far as `serde_yaml` round-trips allow.
   Alternative: fold into `Project` — rejected; package.json vs yarnrc are different documents.

3. **ID string normalization**
   Prefer: store/compare the ID exactly as Yarn emits it in the audit `ID` field (after parsing `serde_json::Value` to a clean string without extra JSON quotes). Glob patterns already in yarnrc are matched against audit IDs the same way Yarn would for membership checks in gnarl's hygiene (exact match first; glob only if we already depend on pattern semantics — prefer exact ID equality for drop decisions unless Yarnrc entries are clearly globs).

4. **Where hygiene runs**
   Prefer: after the main auto install/audit/fix loop settles, and in the same post-loop phase as `reset_resolutions` — read yarnrc + unfiltered audit, drop orphans and within-range-fixable ignores, reset packages for the latter, then if package resets occurred (or resolutions dirty) run install/dedupe as already required for resolution cleanup. `check` (including the final `check` after auto) prints ID hints and the ignore overview for remaining entries.
   Alternative: fold ignore evaluation into the per-advisory fix loop — harder because filtered audit hides ignored IDs.

5. **Overview timing vs drops**
   Prefer: print overview of entries that remain after auto-drop (or of current list during standalone `check`). For standalone `check`, do not mutate yarnrc; only report. Orphans still appear in check overview as unknown package/severity so the user sees them; auto is what removes them.

6. **Within-range criteria reuse**
   Prefer: reuse the same packument / request satisfaction logic `auto` already uses to decide a package is reset-worthy (`has_fix` / within-range), applied to advisories from the unfiltered audit that match ignore IDs.

7. **Severity filtering in gnarl, not Yarn**
   Prefer: run `yarn npm audit --json --recursive` without `--severity`, then apply `-s` as a minimum-severity filter inside gnarl for fix/check. Ignore overview and hygiene use the full (ignore-cleared) set so below-threshold ignored advisories are not mistaken for orphans.
   Alternative considered: pass `--severity` to Yarn — rejected; it hid ignored low/moderate IDs during hygiene when `-s high` was set.

## Risks / Trade-offs

- [Process killed while ignores temporarily cleared for unfiltered audit] → Restore on all controlled paths; document residual risk
- [serde_yaml round-trip reformats `.yarnrc.yml`] → Acceptable; preserve key set and list membership
- [ID rotation: old ID orphan-dropped while new ID still fails audit] → By design for orphans; user may need to add the new ID; within-range auto-fix path still applies to the new advisory once visible
- [False confidence on overview for glob ignore patterns] → Prefer documenting exact IDs; exact match for drop decisions
- [Extra audit cost every check/auto] → One additional unfiltered audit when yarnrc has ignores; skip second audit if ignore list empty

## Migration Plan

- No migration for existing projects: behavior is additive
- Existing `npmAuditIgnoreAdvisories` entries start appearing in overview; orphans and within-range-fixable ones are cleaned on next `gnarl` (auto) run
- Rollback: revert the gnarl version; yarnrc edits already applied remain (user can restore from VCS)

## Open Questions

<!-- none remaining after yarnrc clear/restore decision -->
