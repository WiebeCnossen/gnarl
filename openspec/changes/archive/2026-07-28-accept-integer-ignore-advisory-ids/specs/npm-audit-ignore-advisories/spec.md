## ADDED Requirements

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
