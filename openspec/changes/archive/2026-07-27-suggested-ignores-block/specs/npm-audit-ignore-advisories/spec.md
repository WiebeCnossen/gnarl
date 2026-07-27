## ADDED Requirements

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

## REMOVED Requirements

### Requirement: Mention audit ID for resolution suggestions

**Reason**: Superseded by the dedicated `suggested ignores` section; inline ID on resolution lines blocked copy-paste.
**Migration**: Use `suggested ignores` (enriched lines + YAML) for resolution-candidate IDs; resolution lines stay package.json-ready without annotations.

### Requirement: Mention audit ID when no fix is available

**Reason**: Superseded by the dedicated `suggested ignores` section for unresolved / no-fix cases; inline ID hints removed.
**Migration**: Unresolved candidates appear under `suggested ignores` during `check`; `auto` no-fix messages no longer carry an ID.
