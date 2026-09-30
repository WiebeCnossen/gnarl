# Spec Delta

## MODIFIED Requirements

### Requirement: Suggested ignores section

When `check` (including the final `check` after `auto`) has at least one ignore candidate, gnarl MUST present a section titled `suggested ignores`. Candidates MUST be advisory IDs from outside-range resolution suggestions and from unresolved / no-fix issues. Candidates MUST NOT include deprecations, within-range fix suggestions, or IDs already listed in `.yarnrc.yml` `npmAuditIgnoreAdvisories`. Each candidate MUST appear once. Enrichment lines MUST use the same form as the current `npmAuditIgnoreAdvisories` overview (advisory ID, severity, and package with vulnerable range when known). Immediately after those enrichment lines in the presentation, gnarl MUST provide a YAML block that begins with the `npmAuditIgnoreAdvisories` key and lists only the new suggested IDs (not a merge with existing yarnrc entries), in a form suitable for pasting into `.yarnrc.yml`.

Under tagged-stdout mode (non-TTY or `--raw`), that section and YAML MUST appear on stdout as today. Under the interactive TTY UI, the same enrichment content and YAML MUST appear in the Done-screen next-actions region, and the YAML MUST be the payload offered by the ignores clipboard key when present.

#### Scenario: Suggestions include resolutions and unresolved

- **WHEN** `check` has outside-range resolution candidates and unresolved issues with audit IDs not already ignored, under tagged-stdout mode
- **THEN** stdout MUST include a `suggested ignores` section with enriched lines for those IDs followed by a `npmAuditIgnoreAdvisories:` YAML list of those IDs only

#### Scenario: Suggestions on interactive UI

- **WHEN** `check` (or final `check` after `auto`) has ignore candidates under the interactive TTY UI
- **THEN** the Done screen MUST present the `suggested ignores` enrichment content and the same paste-ready YAML block content as tagged-stdout mode would emit

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

gnarl MUST NOT append `# ignore:` (or equivalent inline audit-ID hints) to resolution suggestion lines, unresolved-issue lines, or `auto` no-fix messages. Audit IDs for those cases MUST appear only via the `suggested ignores` section when applicable. This applies whether those lines are emitted on stdout or shown in the interactive UI.

#### Scenario: Resolution lines are clean

- **WHEN** `check` presents a suggested resolution
- **THEN** the line MUST NOT contain an inline ignore ID annotation

#### Scenario: Unresolved lines are clean

- **WHEN** `check` presents an unresolved issue
- **THEN** the line MUST NOT contain an inline ignore ID annotation

#### Scenario: Auto no-fix message is clean

- **WHEN** `auto` reports that a package has no fix
- **THEN** that message MUST NOT contain an inline ignore ID annotation

### Requirement: Suggested resolutions section title

When `check` presents outside-range resolution suggestions, the section title MUST be `suggested resolutions` (not `resolutions`). Under tagged-stdout mode the title appears on stdout; under the interactive TTY UI it labels the corresponding Done-screen region. Resolution body lines offered for clipboard copy MUST match the tagged-stdout body lines.

#### Scenario: Section renamed

- **WHEN** `check` has at least one outside-range resolution suggestion under tagged-stdout mode
- **THEN** stdout MUST label that section `suggested resolutions`

#### Scenario: Interactive UI uses the same title semantics

- **WHEN** `check` has at least one outside-range resolution suggestion under the interactive TTY UI
- **THEN** the Done screen MUST present those suggestions under the `suggested resolutions` title (or equivalent clear label matching that name)

### Requirement: Overview of current npmAuditIgnoreAdvisories

gnarl MUST present an overview of entries currently listed in `.yarnrc.yml` `npmAuditIgnoreAdvisories`. Enrichment MUST use a severity-unfiltered audit (ignores cleared) so below-`-s` advisories still show package and severity. For each entry that appears in that audit, the overview MUST include the advisory ID, the affected package, and the severity. For orphan entries (ID not present in that audit before drop logic runs), the overview MAY omit package and severity or mark them unknown.

Under tagged-stdout mode the overview MUST appear on stdout. Under the interactive TTY UI it MUST appear in the Done-screen resulting-state region when applicable.

#### Scenario: Enriched overview for known ignores

- **WHEN** `check` or the end of `auto` runs under tagged-stdout mode and `.yarnrc.yml` lists ignore IDs that appear in the severity-unfiltered audit
- **THEN** stdout MUST include a section listing each such ID with package name and severity

#### Scenario: Enriched overview on interactive UI

- **WHEN** `check` or the end of `auto` runs under the interactive TTY UI and `.yarnrc.yml` lists ignore IDs that appear in the severity-unfiltered audit
- **THEN** the Done-screen resulting-state region MUST include each such ID with package name and severity

#### Scenario: Empty ignore list

- **WHEN** `npmAuditIgnoreAdvisories` is missing or empty
- **THEN** gnarl MUST NOT fail and MAY omit the overview section
