# Spec Delta

## ADDED Requirements

### Requirement: Done apply keys for ignores and resolutions

On the interactive Done screen for `auto` and `check`, gnarl MUST accept `I` to apply all current suggested-ignore IDs and `R` to apply all current suggested resolutions. Lowercase `i` and `r` MUST remain clipboard copy. Apply MUST happen without a confirmation dialog. When the corresponding suggestion set is empty, the apply key MUST no-op. `--raw` and non-TTY MUST NOT offer these apply actions.

#### Scenario: I applies ignores on auto Done

- **WHEN** the interactive `auto` Done screen has suggested-ignore YAML and the operator presses `I`
- **THEN** gnarl MUST persist those ignore IDs and MUST NOT treat the key as clipboard copy

#### Scenario: R applies resolutions on check Done

- **WHEN** the interactive `check` Done screen has suggested resolutions and the operator presses `R`
- **THEN** gnarl MUST persist those resolutions and continue into `auto` as specified by apply-suggested-resolutions

#### Scenario: Empty apply is a no-op

- **WHEN** the matching suggestion section is empty and the operator presses `I` or `R`
- **THEN** gnarl MUST NOT write `.yarnrc.yml` or `package.json` and MUST remain on Done

#### Scenario: Raw has no apply keys

- **WHEN** the run uses tagged stdout (`--raw` or non-TTY)
- **THEN** gnarl MUST NOT apply ignores or resolutions from keyboard input

### Requirement: Hints distinguish copy from apply

When copy or apply actions are available on Done, the interactive hints MUST show lowercase `i`/`r` as copy and uppercase `I`/`R` as apply for the sections that have content.

#### Scenario: Both sections populated

- **WHEN** Done has suggested resolutions and suggested-ignore YAML
- **THEN** the hints MUST mention `r`, `i`, `R`, and `I` with copy vs apply distinguished

### Requirement: I refreshes via check without install

After a successful Done apply-ignores write, gnarl MUST re-run `check` (no install or dedupe solely from that yarnrc write) and present a new Done screen. If the session has not switched to the live `auto` dashboard, a session that started as `check` MUST keep the compact report view for that refresh.

#### Scenario: Ignores leave suggested ignores

- **WHEN** the operator applies suggested ignores from Done
- **THEN** the following Done report MUST list those IDs in the current-ignore overview and MUST NOT list them under suggested ignores

#### Scenario: Check session stays compact after I

- **WHEN** the operator started `check` and presses `I` without having applied resolutions
- **THEN** gnarl MUST use the compact report view for the refresh, not the live multi-phase `auto` dashboard

### Requirement: R switches to the live auto dashboard

After the operator applies suggested resolutions from Done, gnarl MUST present the live `auto` dashboard for the continuation `auto`, including when the original verb was `check`. When that `auto` finishes, gnarl MUST wait on a new Done screen for `q` as usual.

#### Scenario: Check then R uses live auto chrome

- **WHEN** interactive `check` reaches Done and the operator applies suggested resolutions
- **THEN** gnarl MUST show the live multi-phase `auto` dashboard for the continuation, not the compact check-only chrome

### Requirement: Quit keys leave immediately

On the interactive TTY UI, `q` MUST restore the terminal and exit the process whether or not the run has reached Done and whether or not a worker is still running. `Esc` MUST do the same, except when a pane is maximized: then `Esc` MUST shrink the pane and MUST NOT exit.

#### Scenario: q during a live auto run

- **WHEN** the interactive `auto` dashboard is still running (not Done) and the user presses `q`
- **THEN** gnarl MUST leave the alternate screen and exit without waiting for Done

#### Scenario: Esc shrinks then quits

- **WHEN** a pane is maximized and the user presses `Esc`
- **THEN** gnarl MUST restore the 2×2 layout and MUST NOT exit

#### Scenario: Esc quits when not maximized

- **WHEN** no pane is maximized (including Done) and the user presses `Esc`
- **THEN** gnarl MUST leave the alternate screen and exit

## MODIFIED Requirements

### Requirement: Compact report view for check

On the interactive TTY UI for `check`, gnarl MUST use a compact report view (not the full live `auto` dashboard) until the operator starts a continuation `auto` from Done. While work is in progress it MAY show a brief computing/status indication. When the report completes, it MUST present the same completion regions as the `auto` Done screen: resulting state and next-action suggestions (when applicable), plus Done-screen controls.

#### Scenario: Check does not use the live auto shell

- **WHEN** the user runs `check` on the interactive TTY UI and has not started a continuation `auto` from Done
- **THEN** gnarl MUST present the compact report view rather than the live multi-phase `auto` dashboard

#### Scenario: Check Done layout matches auto completion regions

- **WHEN** `check` completes on the interactive TTY UI
- **THEN** the view MUST include resulting-state and next-action regions consistent with the `auto` Done screen (content depending on the report)
