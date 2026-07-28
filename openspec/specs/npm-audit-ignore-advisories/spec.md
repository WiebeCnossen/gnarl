## Purpose

Suggest Yarn audit IDs for `npmAuditIgnoreAdvisories` via a dedicated `suggested ignores` section (enriched overview plus paste-ready YAML), overview current ignores with package and severity, and auto-drop orphan or within-range-fixable ignore entries.

## Requirements

### Requirement: Suggested ignores section

When `check` (including the final `check` after `auto`) has at least one ignore candidate, gnarl MUST print a section titled `suggested ignores`. Candidates MUST be advisory IDs from outside-range resolution suggestions and from unresolved / no-fix issues. Candidates MUST NOT include deprecations, within-range fix suggestions, or IDs already listed in `.yarnrc.yml` `npmAuditIgnoreAdvisories`. Each candidate MUST appear once. Enrichment lines MUST use the same form as the current `npmAuditIgnoreAdvisories` overview (advisory ID, severity, and package with vulnerable range when known). Immediately after those lines, gnarl MUST print a YAML block that begins with the `npmAuditIgnoreAdvisories` key and lists only the new suggested IDs (not a merge with existing yarnrc entries), in a form suitable for pasting into `.yarnrc.yml`.

#### Scenario: Suggestions include resolutions and unresolved

- **WHEN** `check` has outside-range resolution candidates and unresolved issues with audit IDs not already ignored
- **THEN** stdout MUST include a `suggested ignores` section with enriched lines for those IDs followed by a `npmAuditIgnoreAdvisories:` YAML list of those IDs only

#### Scenario: Already-ignored IDs omitted

- **WHEN** an advisory would otherwise be a candidate but its ID is already in `npmAuditIgnoreAdvisories`
- **THEN** that ID MUST NOT appear in the enriched lines or the YAML block

#### Scenario: Empty suggestions omitted

- **WHEN** there are no resolution or unresolved candidates, or every such ID is already ignored
- **THEN** gnarl MUST omit the `suggested ignores` section and its YAML block entirely

#### Scenario: Deprecations and within-range fixes excluded

- **WHEN** the only advisories are deprecations and/or within-range fix suggestions
- **THEN** gnarl MUST NOT emit `suggested ignores`

### Requirement: No inline ignore annotations

gnarl MUST NOT append `# ignore:` (or equivalent inline audit-ID hints) to resolution suggestion lines, unresolved-issue lines, or `auto` no-fix messages. Audit IDs for those cases MUST appear only via the `suggested ignores` section when applicable.

#### Scenario: Resolution lines are clean

- **WHEN** `check` prints a suggested resolution
- **THEN** the line MUST NOT contain an inline ignore ID annotation

#### Scenario: Unresolved lines are clean

- **WHEN** `check` prints an unresolved issue
- **THEN** the line MUST NOT contain an inline ignore ID annotation

#### Scenario: Auto no-fix message is clean

- **WHEN** `auto` reports that a package has no fix
- **THEN** that message MUST NOT contain an inline ignore ID annotation

### Requirement: Suggested resolutions section title

When `check` prints outside-range resolution suggestions, the section title MUST be `suggested resolutions` (not `resolutions`).

#### Scenario: Section renamed

- **WHEN** `check` has at least one outside-range resolution suggestion
- **THEN** stdout MUST label that section `suggested resolutions`

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

When running `auto`, if an ignored advisory is **fully** within-range remediable — every relevant requested range of the package has a within-range fix (no blocked sibling ranges) — gnarl MUST remove that advisory's ID from `npmAuditIgnoreAdvisories` and reset the affected package so the update can apply. A merely partial within-range win (some ranges fixable, others blocked) MUST NOT cause the ignore to be dropped, because the advisory is not completely resolved.

#### Scenario: Within-range fix drops ignore and resets package

- **WHEN** an ignore ID appears in the unfiltered audit and every relevant requested range has a within-range update that would fix it
- **THEN** gnarl MUST remove the ID from `.yarnrc.yml`, reset the package in the lockfile/resolutions path used for normal fixes, and treat the tree as dirty for install when installs are enabled

#### Scenario: Outside-range only keeps ignore

- **WHEN** an ignore ID appears in the unfiltered audit and only an outside-range resolution exists
- **THEN** gnarl MUST NOT auto-drop that ignore for the within-range rule

#### Scenario: Partial within-range win keeps ignore

- **WHEN** an ignore ID appears in the unfiltered audit and at least one requested range has a within-range fix while at least one other requested range does not
- **THEN** gnarl MUST leave that ID in `npmAuditIgnoreAdvisories` (even if `auto` resets the package for the partial win under normal fix rules)

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

### Requirement: Accept string or integer ignore advisory IDs

gnarl MUST accept each `.yarnrc.yml` `npmAuditIgnoreAdvisories` list entry whether it is a YAML string scalar or a YAML integer scalar. Both forms MUST be normalized to the same string ID used for overview enrichment, suggested-ignores filtering, remove, and auto-drop. gnarl MUST NOT fail solely because an entry is an unquoted integer. When gnarl writes `npmAuditIgnoreAdvisories`, it MAY emit quoted string form for all entries.

#### Scenario: Unquoted integer entries are readable

- **WHEN** `.yarnrc.yml` contains `npmAuditIgnoreAdvisories` with unquoted integer list items (for example `- 1090865`)
- **THEN** gnarl MUST treat those entries as ignore IDs equivalent to the decimal string form (for example `"1090865"`) and MUST NOT error or crash while reading them

#### Scenario: Mixed string and integer entries

- **WHEN** `npmAuditIgnoreAdvisories` lists both quoted string IDs and unquoted integer IDs
- **THEN** gnarl MUST include all of them in the effective ignore set after normalization

#### Scenario: Integer entry participates in ignore hygiene

- **WHEN** an ignore ID is stored as an unquoted integer and that ID appears in (or is absent from) the severity-unfiltered audit
- **THEN** overview, suggested-ignores omission, and auto-drop MUST behave the same as if the ID had been stored as a string
