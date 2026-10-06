# Spec Delta

## ADDED Requirements

### Requirement: Auto-ignore does not change UI mode

`--auto-ignore` MUST NOT by itself select tagged stdout or the interactive TTY UI. UI mode selection remains: interactive when stdout is a TTY and `--raw` is not set for `auto`/`check`; tagged stdout when stdout is not a TTY or `--raw` is set.

#### Scenario: TTY auto-ignore still uses the dashboard

- **WHEN** `auto --auto-ignore` runs with stdout on a TTY and without `--raw`
- **THEN** gnarl MUST use the interactive TTY UI

#### Scenario: Raw plus auto-ignore stays tagged stdout

- **WHEN** `auto --raw --auto-ignore` runs on a TTY
- **THEN** gnarl MUST use tagged stdout and MUST NOT enter the alternate-screen UI

### Requirement: Interactive quit uses the same policy exit

When the interactive TTY UI reaches a successful Done state for an auto-ignore run, gnarl MUST still wait for the quit key before restoring the terminal. After that quit, the process MUST exit with the same 0 / 10–14 policy status that tagged stdout would have used for that run.

#### Scenario: Quit after applying a critical ignore

- **WHEN** an interactive auto-ignore run newly writes a critical ignore, reaches Done, and the user presses `q`
- **THEN** after restoring the terminal the process MUST exit 14

#### Scenario: Quit with no new ignores

- **WHEN** an interactive auto-ignore run writes no new ignores, reaches Done, and the user presses `q`
- **THEN** after restoring the terminal the process MUST exit 0

### Requirement: Dashboard fix lines include severity for this change

On the interactive live dashboard, advisory-driven reset messages and auto-applied ignore messages MUST include the same severity information required of those messages on tagged stdout.

#### Scenario: Live fixes show ignore severity

- **WHEN** auto-ignore writes a high advisory ID during an interactive `auto` run
- **THEN** the live fixes region MUST include `high` for that applied ignore
