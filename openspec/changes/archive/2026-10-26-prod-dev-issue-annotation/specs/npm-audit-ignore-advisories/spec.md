# Spec Delta

## MODIFIED Requirements

### Requirement: Suggested ignores enrichment and YAML

Enrichment lines in `suggested ignores` MUST include the advisory ID, severity, package with vulnerable range when known, and a trailing environment label `prod` or `dev-only` from the production vs:dev-only classification. Immediately after those lines, gnarl MUST provide a YAML block that begins with the `npmAuditIgnoreAdvisories` key and lists only the new suggested IDs (not a merge with existing yarnrc entries), in a form suitable for pasting into `.yarnrc.yml`. The YAML block MUST NOT include environment labels. The current `npmAuditIgnoreAdvisories` overview enrichment form is unchanged by this requirement.

#### Scenario: Suggestions include resolutions and unresolved

- **WHEN** `check` has outside-range resolution candidates and unresolved issues with audit IDs not already ignored, under tagged-stdout mode
- **THEN** stdout MUST include a `suggested ignores` section with enriched lines for those IDs followed by a `npmAuditIgnoreAdvisories:` YAML list of those IDs only

#### Scenario: Enrichment includes environment label

- **WHEN** `check` presents a suggested-ignore enrichment line for an advisory classified as `dev-only`
- **THEN** that enrichment line MUST end with `dev-only` and the following YAML list entry for that ID MUST be the bare advisory ID
