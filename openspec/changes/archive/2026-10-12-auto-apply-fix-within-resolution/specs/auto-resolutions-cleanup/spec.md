# Spec Delta

## MODIFIED Requirements

### Requirement: Refresh lockfile after dropping unused resolutions

When auto mode removes one or more unused resolutions from `package.json`, gnarl MUST run `yarn install` followed by `yarn dedupe` before the final check. Install-on-change (`-x` / `--install-on-change`) MUST NOT skip that follow-up. After that refresh (and after ignore-hygiene package resets that also refresh), if within-range fixes become available, gnarl MUST re-enter the within-range fix loop and apply them before the final `check`.

#### Scenario: Resolutions dropped with installs enabled

- **WHEN** `reset_resolutions` removes at least one resolution
- **THEN** gnarl runs one `install` and one `dedupe` after saving `package.json` and before `check`

#### Scenario: Resolutions dropped under install-on-change

- **WHEN** `reset_resolutions` removes at least one resolution and `-x` or `--install-on-change` is set
- **THEN** gnarl MUST still run the follow-up `install` and `dedupe` after saving `package.json`

#### Scenario: No resolutions dropped

- **WHEN** `reset_resolutions` leaves all resolutions in place
- **THEN** gnarl MUST NOT run an extra `install` or `dedupe` solely for resolution cleanup

#### Scenario: Hygiene unlocks a within-range fix

- **WHEN** dropping unused resolutions or ignore hygiene refreshes the tree and a within-range fix is then available
- **THEN** `auto` MUST apply that fix (re-enter the within-range reset loop) before the final `check`
