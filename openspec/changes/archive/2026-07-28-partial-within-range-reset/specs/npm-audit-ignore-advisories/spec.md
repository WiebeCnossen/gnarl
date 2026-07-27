## MODIFIED Requirements

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
