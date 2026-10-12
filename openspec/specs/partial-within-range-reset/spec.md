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

### Requirement: Apply within-range fix admitted by an existing resolution

When running `auto`, if `package.json` already has a resolution for a package and a newer safe version satisfies that resolution’s range (and remediates the advisory), gnarl MUST apply that update so the lockfile moves to the safe version. The existing resolution MUST NOT leave a within-range remediable advisory for the final `check`.

#### Scenario: Caret resolution admits a newer safe version

- **WHEN** `package.json` has a resolution whose range admits a newer non-vulnerable version than the locked version (even if dependents still declare an older exact range that the resolution remaps)
- **THEN** `auto` MUST reset/re-resolve that package so the lockfile lands on a safe version within the resolution range

#### Scenario: Exact pin below a within-range safe version is widened

- **WHEN** an existing resolution pins an exact vulnerable version and a newer safe version would satisfy the dependency’s requested range
- **THEN** `auto` MUST update or replace that resolution so the safe within-range version can apply, then refresh the lockfile accordingly

### Requirement: Auto finishes with no within-range fixes left

After `auto` completes its within-range work (including any post-hygiene re-entry), the final `check` MUST NOT list any entries in the `fixes` section. Remaining work MAY still appear as suggested resolutions, suggested ignores, unresolved issues, or deprecations.

#### Scenario: Final check has empty fixes

- **WHEN** `auto` finishes and emits its final report
- **THEN** the `fixes` section MUST be absent or empty

#### Scenario: Suggested resolutions may remain

- **WHEN** `auto` finishes and some advisories still need an outside-range resolution
- **THEN** those MAY appear under `suggested resolutions`, and MUST NOT appear under `fixes`

### Requirement: Prefer within-range fix over suggested resolution when classifying

When classifying an advisory for `check` (including the final `check` after `auto`), if the same package key has both a within-range fix and an outside-range suggested resolution, gnarl MUST keep the within-range fix and MUST NOT suppress it in favor of the suggested resolution.

#### Scenario: Same key classifies as fix not resolution

- **WHEN** classification finds both a within-range fix and an outside-range resolution candidate for the same package key
- **THEN** the report MUST list that key under `fixes` and MUST NOT list it under `suggested resolutions`
