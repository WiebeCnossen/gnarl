# Tasks

## 1. Structured suggestion snapshots

- [x] 1.1 Add reporter events for suggested-resolution entries (key + version) and suggested-ignore IDs emitted with the existing report sections; verify `StdoutReporter` still emits the same tagged section/YAML text and ignores the structured events
- [x] 1.2 Store those snapshots on `UiState` from the events and keep copy payloads on formatted strings; verify unit tests that the snapshot keys match `format_resolution_line` / ignore IDs and empty sections have empty snapshots

## 2. Ignore apply without policy

- [x] 2.1 Split yarnrc suggested-ignore merge from policy `RunStatus` so a helper writes IDs from a snapshot (or classified list) and returns wrote + max severity; verify `--auto-ignore` still maps that helper to 10–14 and existing auto-ignore tests pass
- [x] 2.2 Add `Gnarl` apply-ignores-then-`check` that uses the helper and always returns success `RunStatus::ok()` (no 10–14); verify unit tests that IDs are merged, install/dedupe are not invoked, and status stays 0 even for critical IDs

## 3. Resolution apply then auto

- [x] 3.1 Implement writing snapshot resolutions via `Project::set_resolution` (`^{version}`) and `save`, keeping unrelated entries; verify unit tests for insert, keep-existing, and empty snapshot does not write
- [x] 3.2 Chain that write into `Gnarl::auto(true)` with the caller’s `Options`; verify a unit/harness test that `refresh_first` is true (install-on-change still opens with refresh) and `check` options do not set `auto_ignore`

## 4. Interactive session loop

- [x] 4.1 Allow `run_interactive` to spawn a follow-up worker on `I`/`R` without leaving the alternate screen, one worker at a time; verify a test or small harness that a second work cycle can run after `done` and that apply keys no-op while a worker is running
- [x] 4.2 Map `KeyCode::Char('I')` / `'R'` to apply (lowercase stays copy), no-op when the snapshot is empty; verify state/event-loop tests for copy vs apply, empty no-op, and `check` + `R` sets `verb` to `Auto` so `is_live_auto()` is true
- [x] 4.3 Update Done hints to distinguish `[i]`/`[r]` copy from `[I]`/`[R]` apply when those sections have content; verify `keyboard_hints` tests for both sections populated and for empty
- [x] 4.4 After `I`, clear report Done flags and keep compact `check` chrome unless already live auto; after `R`, switch to live auto chrome; verify `UiState` tests for verb/layout after those transitions
- [x] 4.5 Combine session `RunStatus` as max of `--auto-ignore` persists only (`I` never updates it; later auto with no new IDs keeps earlier 10–14); verify unit tests of that merge and that `finish_interactive` returns the session status after `q`

## 5. Docs

- [x] 5.1 Document TTY Done keys `i`/`r` copy and `I`/`R` apply (and that `R` runs full `auto`) in README and `gnarl help`; verify help/README tests or string assertions mention `I` and `R`

## 6. Integration

- [x] 6.1 Run `cargo clippy --all-targets` and clear every warning
