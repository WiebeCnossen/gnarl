## ADDED Requirements

### Requirement: Refresh lockfile after dropping unused resolutions

When auto mode removes one or more unused resolutions from `package.json`, gnarl MUST run `yarn install` followed by `yarn dedupe` once before the final check, unless installs are disabled.

#### Scenario: Resolutions dropped with installs enabled

- **WHEN** `reset_resolutions` removes at least one resolution and `--no-install` is not set
- **THEN** gnarl runs one `install` and one `dedupe` after saving `package.json` and before `check`

#### Scenario: Resolutions dropped with installs disabled

- **WHEN** `reset_resolutions` removes at least one resolution and `--no-install` is set
- **THEN** gnarl MUST NOT run the follow-up `install` or `dedupe`

#### Scenario: No resolutions dropped

- **WHEN** `reset_resolutions` leaves all resolutions in place
- **THEN** gnarl MUST NOT run an extra `install` or `dedupe` solely for resolution cleanup
