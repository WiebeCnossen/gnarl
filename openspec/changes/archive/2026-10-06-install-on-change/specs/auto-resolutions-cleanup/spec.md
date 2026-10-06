# Spec Delta

## MODIFIED Requirements

### Requirement: Refresh lockfile after dropping unused resolutions

When auto mode removes one or more unused resolutions from `package.json`, gnarl MUST run `yarn install` followed by `yarn dedupe` once before the final check. Install-on-change (`-x` / `--install-on-change`) MUST NOT skip that follow-up.

#### Scenario: Resolutions dropped with installs enabled

- **WHEN** `reset_resolutions` removes at least one resolution
- **THEN** gnarl runs one `install` and one `dedupe` after saving `package.json` and before `check`

#### Scenario: Resolutions dropped under install-on-change

- **WHEN** `reset_resolutions` removes at least one resolution and `-x` or `--install-on-change` is set
- **THEN** gnarl MUST still run the follow-up `install` and `dedupe` after saving `package.json`

#### Scenario: No resolutions dropped

- **WHEN** `reset_resolutions` leaves all resolutions in place
- **THEN** gnarl MUST NOT run an extra `install` or `dedupe` solely for resolution cleanup
