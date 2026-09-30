# Tasks

## 1. Presentation boundary

- [x] 1.1 Define `UiEvent` / `Reporter` API (activity, fix, phase, report sections, error) in a new module and verify it compiles with a no-op or collecting test double
- [x] 1.2 Implement `StdoutReporter` that maps events to today’s `[YARN]`/`[HIT#]`/`[INFO]`/`[FIX!]`/`[NPM?]` tags, `print_section`, KPIs, and bare ignore YAML, and verify a unit/integration capture matches current tag and section layout for a fixture report
- [x] 1.3 Route emitters in `Gnarl`, `Yarn`, `Locks`, and `Npm` through `Reporter` (retire inline `out_*!` as the primary path) and verify existing tests pass under tagged-stdout behavior
- [x] 1.4 Extract freestanding formatters for suggested-resolution body lines and ignore YAML (shared by stdout + clipboard) and verify formatter unit tests match current string shapes

## 2. CLI mode selection

- [x] 2.1 Add `--raw` to `Options` / `Command` parsing and verify parse tests accept `--raw` alongside `-x` / `-s`
- [x] 2.2 Implement UI mode selection (interactive iff TTY && !raw && verb is `auto` or `check`) and verify unit tests for TTY/raw/verb combinations
- [x] 2.3 Wire `main` to choose stdout vs interactive entry for `auto`/`check`, keep standalone `reset`/`help`/`info` on stdout, and verify help text documents `--raw`

## 3. Interactive runtime shell

- [x] 3.1 Add `ratatui`, `crossterm`, and clipboard (`arboard` or equivalent) dependencies and verify `cargo check` succeeds
- [x] 3.2 Implement terminal enter/leave (alt screen, raw mode) with restore-on-drop and verify a smoke test or manual checklist leaves the terminal usable after quit
- [x] 3.3 Implement worker-thread run + `UiEvent` channel + crossterm input loop skeleton and verify the UI thread keeps redrawing while the worker blocks on a test sleep/event

## 4. Compact check TUI + Done

- [x] 4.1 Build shared `UiState` Done regions (STATE + NEXT + key hints) fed by report events and verify state unit tests populate KPIs, ignores, resolutions, and ignore YAML from events
- [x] 4.2 Implement compact `check` view (computing → Done) on the interactive path and verify interactive `check` reaches Done with report content (manual or harness)
- [x] 4.3 Implement always-block-until-`q` on successful Done and verify the process does not exit before `q` in an interactive harness or documented manual check
- [x] 4.4 Implement clipboard keys for suggested resolutions and ignore YAML via shared formatters and verify copied strings equal formatter output (mock clipboard or integration assert)

## 5. Live auto dashboard

- [x] 5.1 Add live ACTIVITY + FIXED panels and phase/elapsed chrome for interactive `auto`, keeping STATE/NEXT empty until report completion, and verify pre-report state tests omit completed report panels while recording activity/fixes
- [x] 5.2 Hook interactive `auto` (including `reset`→`auto`) through the worker + live dashboard and verify a manual or harness run shows live updates then Done with quit/copy
- [x] 5.3 Ensure blocked-by uniqueness still applies on the activity channel (stdout and TUI log) and verify the existing uniqueness scenarios still hold under both backends

## 6. Errors and polish

- [x] 6.1 Implement TUI error modal, dismiss/quit, non-zero exit, and terminal restore; verify a forced worker error shows the modal and restores the terminal
- [x] 6.2 Confirm `--raw` and non-TTY paths never enter alt screen and still exit without requiring `q`; verify with redirected stdout / `--raw` runs
- [x] 6.3 Run `cargo clippy --all-targets` and `cargo test`, fixing warnings/failures until clean
