# tty-dashboard Specification

## Purpose

Present gnarl’s run activity, applied fixes, resulting state, and pasteable next actions on an interactive TTY dashboard (live for `auto`, compact for `check`), with tagged-stdout fallback for non-TTY and `--raw`.

## Requirements

### Requirement: UI mode selection

When stdout is connected to an interactive TTY and `--raw` is not set, `auto` and `check` MUST use the interactive TTY UI. When stdout is not a TTY, or when `--raw` is set, gnarl MUST use the tagged-stdout presentation path (equivalent to today’s tags and section layout) and MUST NOT enter an alternate-screen interactive UI. `--raw` MUST be accepted as a CLI option alongside existing options.

#### Scenario: TTY without raw uses interactive UI

- **WHEN** the user runs `auto` or `check` with stdout attached to a TTY and without `--raw`
- **THEN** gnarl MUST present output via the interactive TTY UI for that verb

#### Scenario: Non-TTY uses tagged stdout

- **WHEN** the user runs `auto` or `check` with stdout not attached to a TTY
- **THEN** gnarl MUST emit the tagged-stdout presentation and MUST NOT require a quit key to exit after a successful run

#### Scenario: --raw forces tagged stdout on a TTY

- **WHEN** the user runs `auto` or `check` on a TTY with `--raw`
- **THEN** gnarl MUST emit the tagged-stdout presentation as if non-TTY

### Requirement: Live dashboard for auto

On the interactive TTY UI for `auto`, gnarl MUST present a live dashboard that updates during the run with (1) current activity information and (2) information about fixes applied during the run. KPIs, existing-ignore overview, and next-action suggestion sections MUST NOT appear as completed report content until the final report phase completes.

#### Scenario: Activity and fixes update before report

- **WHEN** `auto` is running on the interactive TTY UI before the final report completes
- **THEN** the dashboard MUST show updating activity information and MUST accumulate applied-fix information as fixes occur, without showing the completed STATE / next-actions report panels

#### Scenario: Report panels appear only on completion

- **WHEN** the final report phase of `auto` completes on the interactive TTY UI
- **THEN** the dashboard MUST show resulting-state information (including KPIs and existing ignores when applicable) and next-action suggestions when applicable

### Requirement: Compact report view for check

On the interactive TTY UI for `check`, gnarl MUST use a compact report view (not the full live `auto` dashboard). While work is in progress it MAY show a brief computing/status indication. When the report completes, it MUST present the same completion regions as the `auto` Done screen: resulting state and next-action suggestions (when applicable), plus Done-screen controls.

#### Scenario: Check does not use the live auto shell

- **WHEN** the user runs `check` on the interactive TTY UI
- **THEN** gnarl MUST present the compact report view rather than the live multi-phase `auto` dashboard

#### Scenario: Check Done layout matches auto completion regions

- **WHEN** `check` completes on the interactive TTY UI
- **THEN** the view MUST include resulting-state and next-action regions consistent with the `auto` Done screen (content depending on the report)

### Requirement: Always block until quit on interactive Done

When the interactive TTY UI for `auto` or `check` reaches a successful Done state, gnarl MUST wait for an explicit quit key (`q`) before restoring the terminal and exiting, whether or not any suggestions are present.

#### Scenario: Quit required with suggestions

- **WHEN** the interactive UI reaches Done and next-action suggestions are present
- **THEN** gnarl MUST remain on the Done screen until the user presses `q`

#### Scenario: Quit required without suggestions

- **WHEN** the interactive UI reaches Done and there are no next-action suggestions
- **THEN** gnarl MUST still remain on the Done screen until the user presses `q`

### Requirement: Clipboard copy of suggestion payloads

On the interactive Done screen, when suggested resolution lines are present, gnarl MUST offer a keybinding that copies those lines to the system clipboard using the **same text** that the tagged-stdout path would emit for the `suggested resolutions` body lines. When suggested-ignore YAML is present, gnarl MUST offer a keybinding that copies the **same** paste-ready `npmAuditIgnoreAdvisories` YAML block the tagged-stdout path would emit. Keys MUST no-op or be unavailable when the corresponding section is empty.

#### Scenario: Copy resolutions matches stdout fragments

- **WHEN** the Done screen has suggested resolution lines and the user activates the resolutions copy key
- **THEN** the clipboard MUST contain exactly those resolution body lines as emitted on the tagged-stdout path (no UI chrome)

#### Scenario: Copy ignores matches pretty ignore YAML

- **WHEN** the Done screen has a suggested-ignores YAML block and the user activates the ignores copy key
- **THEN** the clipboard MUST contain exactly that YAML block as emitted on the tagged-stdout path

#### Scenario: Empty section does not copy

- **WHEN** the corresponding suggestion section is empty
- **THEN** activating that copy key MUST NOT write an unrelated payload (no-op or key unavailable)

### Requirement: TUI error modal

When an error occurs while the interactive TTY UI is active, gnarl MUST show an error modal in the UI, allow the user to dismiss or quit, and MUST restore the terminal to a usable state afterward. This requirement does not change the non-interactive failure path (tagged stdout / normal process error reporting).

#### Scenario: Error while dashboard active

- **WHEN** a run fails after the interactive TTY UI has been entered
- **THEN** gnarl MUST display the error in a modal and MUST leave the terminal restored after the user dismisses or quits

### Requirement: Reset without auto stays tagged stdout

When the user runs `reset` and that invocation does not proceed into `auto`, gnarl MUST use the tagged-stdout presentation path even on a TTY (no interactive dashboard solely for that reset). When `reset` continues into `auto`, UI mode selection for `auto` applies as usual.

#### Scenario: Standalone reset is stdout

- **WHEN** the user runs `reset` with packages and the run does not chain into `auto`
- **THEN** gnarl MUST emit tagged-stdout style output and MUST NOT enter the interactive dashboard only for that reset

#### Scenario: Reset chaining into auto uses auto UI mode

- **WHEN** `reset` chains into `auto` and UI mode selection chooses the interactive UI
- **THEN** the subsequent `auto` portion MUST use the live dashboard

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
