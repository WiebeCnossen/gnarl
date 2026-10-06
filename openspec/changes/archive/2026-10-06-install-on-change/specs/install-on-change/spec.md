# Spec Delta

## Purpose

Lets `auto` skip a speculative opening Yarn refresh when `-x` / `--install-on-change` is set, while still running install+dedupe after this process mutates `package.json` or `yarn.lock`.

## ADDED Requirements

### Requirement: Install-on-change flag aliases

gnarl MUST accept `-x` and `--install-on-change` as the same option. Default `auto` (flag absent) MUST still open with `yarn install` then `yarn dedupe` before the first audit.

#### Scenario: Short and long form are equivalent

- **WHEN** the user passes `-x` or `--install-on-change` on `auto` (including default auto) or `reset`
- **THEN** gnarl MUST treat both as install-on-change and MUST NOT fail solely because the flag is present

#### Scenario: Default auto still opens with refresh

- **WHEN** `auto` runs without `-x` and without `--install-on-change`
- **THEN** gnarl MUST run `yarn install` followed by `yarn dedupe` before the first audit of that run

### Requirement: Help and README document install-on-change

`gnarl help` MUST name both `-x` and `--install-on-change` and describe that they skip the opening install+dedupe while still refreshing after this-run `package.json` or `yarn.lock` mutations. The README MUST document the same option and MUST present `gnarl auto --raw --auto-ignore --install-on-change` as bot best practice.

#### Scenario: Help names both spellings

- **WHEN** the user runs `gnarl help`
- **THEN** the help text MUST include `-x` and `--install-on-change`

#### Scenario: README states bot best practice

- **WHEN** a reader follows the README for unattended / bot `auto`
- **THEN** the documented command MUST be `gnarl auto --raw --auto-ignore --install-on-change`

### Requirement: Skip opening refresh on install-on-change auto

When `auto` runs with install-on-change and this process has not yet mutated `package.json` or `yarn.lock`, gnarl MUST NOT run the opening `yarn install` or `yarn dedupe` before the first audit.

#### Scenario: Clean auto skips opening install

- **WHEN** `auto -x` starts with no prior mutation of `package.json` or `yarn.lock` in this process
- **THEN** gnarl MUST begin with audit (and subsequent auto work) and MUST NOT run install or dedupe before that first audit

### Requirement: Refresh after this-run package.json or yarn.lock mutations

When install-on-change is set, gnarl MUST still run `yarn install` then `yarn dedupe` after this process mutates `package.json` (unused resolutions dropped) or `yarn.lock` (within-range reset, ignore-hygiene package reset, or standalone `reset`). Yarnrc-only writes MUST NOT by themselves trigger that refresh. Mutations from before the process started MUST NOT count.

#### Scenario: Within-range lockfile reset still refreshes

- **WHEN** `auto -x` applies a within-range lockfile reset
- **THEN** gnarl MUST run `yarn install` and `yarn dedupe` after that reset and MUST continue the auto loop (further audit) instead of stopping because the flag is set

#### Scenario: Unused resolutions drop still refreshes

- **WHEN** `auto -x` removes at least one unused resolution from `package.json`
- **THEN** gnarl MUST run `yarn install` and `yarn dedupe` after saving `package.json`

#### Scenario: Yarnrc-only write does not refresh

- **WHEN** the only file this run writes is `.yarnrc.yml` (including `--auto-ignore` merges or orphan-only ignore drops)
- **THEN** gnarl MUST NOT run install or dedupe solely because yarnrc changed

#### Scenario: Pre-existing edits are ignored

- **WHEN** `package.json` or `yarn.lock` was edited before gnarl started and `auto -x` does not mutate them
- **THEN** gnarl MUST NOT treat those prior edits as a reason to install or dedupe

### Requirement: Reset chains into auto under install-on-change

When `reset` dirties `yarn.lock`, gnarl MUST chain into `auto` even if install-on-change is set. That chained auto MUST run `yarn install` then `yarn dedupe` before its first audit.

#### Scenario: Reset with -x refreshes then autos

- **WHEN** `gnarl reset -x` successfully resets at least one package in the lockfile
- **THEN** gnarl MUST chain into `auto` and MUST run install and dedupe before that auto’s first audit

#### Scenario: Reset with no lockfile change does not force refresh

- **WHEN** `gnarl reset -x` does not change `yarn.lock`
- **THEN** gnarl MUST NOT chain into auto solely for the flag and MUST NOT run install or dedupe solely because reset was invoked
