# Spec Delta

## ADDED Requirements

### Requirement: Auto-ignore writes follow existing yarnrc rules

When `auto --auto-ignore` persists suggested-ignore candidate IDs, gnarl MUST merge those IDs into `npmAuditIgnoreAdvisories` using the same ID normalization, quoted-string write form, and line-ending preservation as other yarnrc ignore saves. It MUST NOT replace the list with only the new IDs. Candidate membership remains the suggested-ignores rules (outside-range and unresolved / no-fix; not deprecations, within-range fixes, or IDs already listed).

#### Scenario: Existing ignores are kept

- **WHEN** `.yarnrc.yml` already lists ignore ID `1111111` and auto-ignore adds `2222222`
- **THEN** the saved `npmAuditIgnoreAdvisories` list MUST contain both IDs

#### Scenario: CRLF yarnrc stays CRLF

- **WHEN** auto-ignore saves new ignore IDs into an existing `.yarnrc.yml` that uses CRLF line endings
- **THEN** the rewritten ignore block MUST use CRLF

#### Scenario: Already-listed candidate is not duplicated

- **WHEN** a suggested-ignore candidate ID is already present in `npmAuditIgnoreAdvisories` after hygiene
- **THEN** gnarl MUST NOT append a second copy of that ID
