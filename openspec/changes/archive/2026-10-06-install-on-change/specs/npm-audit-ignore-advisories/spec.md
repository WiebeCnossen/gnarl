# Spec Delta

## MODIFIED Requirements

### Requirement: Refresh after ignore-list mutations that change the tree

When `auto` drops ignore entries that also trigger package resets, gnarl MUST run `yarn install` followed by `yarn dedupe` so the lockfile reflects the updates. Orphan-only drops that do not reset packages do not require install solely for the yarnrc edit. Install-on-change (`-x` / `--install-on-change`) MUST NOT skip that follow-up after a package reset.

#### Scenario: Install after drop-with-reset

- **WHEN** at least one ignore is dropped because a within-range fix is available
- **THEN** gnarl MUST run `yarn install` and `yarn dedupe` after applying the resets

#### Scenario: Install after drop-with-reset under install-on-change

- **WHEN** at least one ignore is dropped because a within-range fix is available and `-x` or `--install-on-change` is set
- **THEN** gnarl MUST still run `yarn install` and `yarn dedupe` after applying the resets

#### Scenario: No install for orphan-only yarnrc cleanup

- **WHEN** the only ignore changes are orphan ID removals and no package was reset
- **THEN** gnarl MUST NOT run install solely because `.yarnrc.yml` changed
