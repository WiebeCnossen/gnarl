## ADDED Requirements

### Requirement: Mention audit ID for resolution suggestions

When `check` reports a resolution suggestion for an advisory (fix exists only outside the requested range), gnarl MUST also print the advisory ID as reported by `yarn npm audit`, as a candidate for `npmAuditIgnoreAdvisories`.

#### Scenario: Resolution suggestion includes ID

- **WHEN** `check` classifies an advisory as needing a `package.json` resolution
- **THEN** the output for that suggestion MUST include the advisory's audit ID

#### Scenario: Deprecations excluded

- **WHEN** the advisory is a deprecation
- **THEN** gnarl MUST NOT suggest an `npmAuditIgnoreAdvisories` ID for it

### Requirement: Mention audit ID when no fix is available

When `check` reports an unresolved issue (no fix in the packument for the vulnerable range) or `auto` reports that a package has no fix, gnarl MUST also print the advisory ID as reported by `yarn npm audit`, as a candidate for `npmAuditIgnoreAdvisories`.

#### Scenario: Unresolved issue includes ID

- **WHEN** `check` classifies an advisory as an unresolved issue
- **THEN** the output for that issue MUST include the advisory's audit ID

#### Scenario: Auto no-fix message includes ID

- **WHEN** `auto` reports that a package has no fix
- **THEN** that message MUST include the advisory's audit ID

### Requirement: Overview of current npmAuditIgnoreAdvisories

gnarl MUST print an overview of entries currently listed in `.yarnrc.yml` `npmAuditIgnoreAdvisories`. Enrichment MUST use a severity-unfiltered audit (ignores cleared) so below-`-s` advisories still show package and severity. For each entry that appears in that audit, the overview MUST include the advisory ID, the affected package, and the severity. For orphan entries (ID not present in that audit before drop logic runs), the overview MAY omit package and severity or mark them unknown.

#### Scenario: Enriched overview for known ignores

- **WHEN** `check` or the end of `auto` runs and `.yarnrc.yml` lists ignore IDs that appear in the severity-unfiltered audit
- **THEN** stdout MUST include a section listing each such ID with package name and severity

#### Scenario: Empty ignore list

- **WHEN** `npmAuditIgnoreAdvisories` is missing or empty
- **THEN** gnarl MUST NOT fail and MAY omit the overview section

### Requirement: Auto-drop orphan ignore entries

When running `auto`, gnarl MUST remove from `.yarnrc.yml` `npmAuditIgnoreAdvisories` any entry whose ID does not appear in a severity-unfiltered audit (ignores cleared; all severities included). Absence due only to the `-s` severity threshold MUST NOT count as orphaned.

#### Scenario: Orphan ID dropped

- **WHEN** an ignore ID is present in `.yarnrc.yml` but absent from the severity-unfiltered audit
- **THEN** gnarl MUST remove that ID from `npmAuditIgnoreAdvisories` and save `.yarnrc.yml`

#### Scenario: Below-threshold ignore is not treated as orphan

- **WHEN** an ignore ID appears only as a below-`-s`-threshold advisory in the full audit
- **THEN** gnarl MUST NOT drop that ID as an orphan

#### Scenario: Present ID kept when not within-range fixable

- **WHEN** an ignore ID appears in the severity-unfiltered audit and no within-range fix is available
- **THEN** gnarl MUST leave that ID in `npmAuditIgnoreAdvisories`

### Requirement: Auto-drop ignores superseded by within-range fixes

When running `auto`, if an ignored advisory has a within-range fix available (same criteria gnarl uses to reset a package for a normal advisory), gnarl MUST remove that advisory's ID from `npmAuditIgnoreAdvisories` and reset the affected package so the update can apply.

#### Scenario: Within-range fix drops ignore and resets package

- **WHEN** an ignore ID appears in the unfiltered audit and a within-range update would fix it
- **THEN** gnarl MUST remove the ID from `.yarnrc.yml`, reset the package in the lockfile/resolutions path used for normal fixes, and treat the tree as dirty for install when installs are enabled

#### Scenario: Outside-range only keeps ignore

- **WHEN** an ignore ID appears in the unfiltered audit and only an outside-range resolution exists
- **THEN** gnarl MUST NOT auto-drop that ignore for the within-range rule

### Requirement: Refresh after ignore-list mutations that change the tree

When `auto` drops ignore entries that also trigger package resets, and installs are enabled, gnarl MUST run `yarn install` followed by `yarn dedupe` so the lockfile reflects the updates. Orphan-only drops that do not reset packages do not require install solely for the yarnrc edit.

#### Scenario: Install after drop-with-reset

- **WHEN** at least one ignore is dropped because a within-range fix is available and `--no-install` is not set
- **THEN** gnarl MUST run `yarn install` and `yarn dedupe` after applying the resets

#### Scenario: No install for orphan-only yarnrc cleanup

- **WHEN** the only ignore changes are orphan ID removals and no package was reset
- **THEN** gnarl MUST NOT run install solely because `.yarnrc.yml` changed

#### Scenario: No install when disabled

- **WHEN** ignore hygiene would otherwise reset packages but `--no-install` is set
- **THEN** gnarl MUST still update `.yarnrc.yml` (and perform resets as today when no-install allows) but MUST NOT run the follow-up install/dedupe
