## Purpose

When `auto` finds a within-range fix for at least one requested range of a package, reset that package so the partial update applies, even if other ranges of the same package remain blocked.

## Requirements

### Requirement: Reset on any within-range win

When running `auto`, if at least one requested range of an advisory’s package has a within-range fix available, gnarl MUST reset that package so the update can apply, even when one or more other requested ranges of the same package do not have a within-range fix (non-overlapping / blocked ranges).

#### Scenario: Mixed fixable and blocked ranges trigger reset

- **WHEN** a package has two or more requested ranges in the lockfile for the same advisory module, at least one range has a within-range fix, and at least one other range does not
- **THEN** gnarl MUST reset the package (same lockfile reset path as a fully fixable package) and treat the tree as dirty for install when installs are enabled

#### Scenario: Fully fixable package still resets

- **WHEN** every relevant requested range of the package has a within-range fix
- **THEN** gnarl MUST still reset the package as before

#### Scenario: Blocked-only package does not reset

- **WHEN** no requested range of the package has a within-range fix
- **THEN** gnarl MUST NOT reset the package solely for that advisory

### Requirement: Escalate blocked ranges alongside partial reset

When a package is reset because of a partial within-range win, gnarl MUST still apply existing blocked-range handling for ranges without a within-range fix (including creating parent advisories for npm dependents when applicable).

#### Scenario: Partial reset does not skip parent escalation

- **WHEN** `auto` resets a package due to a within-range win on some ranges and other ranges are blocked by npm dependents
- **THEN** gnarl MUST still enqueue parent advisories for those blocked dependents as it does today when ranges are blocked
