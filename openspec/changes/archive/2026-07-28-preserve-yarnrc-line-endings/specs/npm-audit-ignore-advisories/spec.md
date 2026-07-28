## ADDED Requirements

### Requirement: Preserve yarnrc line endings on save

When gnarl writes or updates `.yarnrc.yml`, it MUST use the same line-ending convention already present in that file. If the existing file uses CRLF (`\r\n`), the rewritten `npmAuditIgnoreAdvisories` block and any newlines gnarl inserts while splicing MUST use CRLF. If the existing file uses LF (`\n`) only, gnarl MUST continue to write LF. When creating a new `.yarnrc.yml` because none existed, gnarl MAY write LF. gnarl MUST NOT convert the entire file's untouched regions solely to change line endings.

#### Scenario: Save preserves CRLF yarnrc

- **WHEN** an existing `.yarnrc.yml` uses CRLF line endings and gnarl saves a change to `npmAuditIgnoreAdvisories`
- **THEN** the written file MUST use CRLF for the rewritten ignore block and MUST NOT introduce LF-only newlines in that block

#### Scenario: Save preserves LF yarnrc

- **WHEN** an existing `.yarnrc.yml` uses LF line endings and gnarl saves a change to `npmAuditIgnoreAdvisories`
- **THEN** the written file MUST continue to use LF line endings

#### Scenario: New yarnrc defaults to LF

- **WHEN** `.yarnrc.yml` does not exist and gnarl creates it with ignore entries
- **THEN** the new file MAY use LF line endings
