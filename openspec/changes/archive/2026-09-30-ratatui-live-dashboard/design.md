# Design

## Context

Today all UX is six `out_*!` macros in `ux.rs` plus `print_section` / `Kpis::print` / bare `print!` for ignore YAML. Emitters are inline across `Gnarl`, `Yarn`, `Locks`, and `Npm`. There is no reporter trait, no TTY detection, and no clipboard dependency. CLI options are hand-parsed in `cmd.rs` (`-x`, `-s`). See proposal.md for motivation and locked product decisions.

## Goals / Non-Goals

**Goals:**

- Split presentation from business logic via a single event/reporter path with two backends (tagged stdout vs interactive TTY).
- Keep tagged-stdout output byte-compatible for scripts and `--raw` (same tags, section titles, resolution fragments, ignore YAML).
- Run blocking yarn/npm work without freezing the TUI (worker + channel, or equivalent).
- Clean terminal restore on quit and after error modal.

**Non-Goals:**

- Streaming or parsing yarn’s own progress UI into the activity panel (status line / spinner for long subprocesses is enough).
- Interactive editing of `package.json` / `.yarnrc.yml` from the dashboard.
- Redesigning `help` / `info`, or a dedicated TUI for standalone `reset`.
- Merging the final `check` I/O into the last `auto` loop (report may still re-audit / re-query as today).
- Mouse-driven copy or dump-on-exit scrollback (clipboard keys only).

## Decisions

### 1. Reporter + `UiEvent` as the UX boundary

**Choice:** Replace direct `out_*!` side effects in library paths with a `Reporter` (or callback/channel) that accepts structured events: activity (yarn/npm/info/hit), fix applied, phase change, report sections (KPIs, ignores, fixes, suggested resolutions, unresolved, suggested ignores + YAML), and fatal error. `StdoutReporter` maps events to today’s tagged lines. TUI backends fold events into `UiState` and redraw.

**Alternatives considered:** Keep macros and tee to a log buffer (weak structure, hard to build panels); full async rewrite with tracing subscribers (heavier than needed).

### 2. UI mode gate in `main` / command options

**Choice:** `Options` gains `raw: bool` from `--raw`. Effective mode = `Interactive` iff stdout is a TTY and `!raw` and verb is `Auto` or `Check` (including `auto` after `reset`). Otherwise `Stdout`.

**Alternatives considered:** Env var only (less discoverable); separate `gnarl ui` subcommand (splits workflows users already have).

### 3. Sync worker thread + crossterm event loop

**Choice:** Keep business logic synchronous. For interactive mode, spawn the run on a worker thread; main thread owns terminal, receives `UiEvent` on a channel, polls crossterm input, redraws with ratatui. On Done, stop accepting work events and only handle `q` / copy keys until quit.

**Alternatives considered:** `tokio` + async yarn wrappers (large migration); redraw only between phases (TUI freezes during `yarn install` — rejects “live”).

### 4. Two TUI compositions, one Done model

**Choice:** Shared `UiState` / Done layout for STATE + NEXT + key hints. `auto` adds live ACTIVITY + FIXED panels and phase/elapsed chrome. `check` uses compact chrome (computing → Done) without the live fix ticker.

**Alternatives considered:** One full dashboard for both (noisier for `check`); entirely separate state machines (duplicated Done/copy/error behavior).

### 5. Clipboard payloads from freestanding formatters

**Choice:** Build resolution body lines and ignore YAML via the same helpers used by `StdoutReporter` (today: resolution `format!("\"{}\": \"^{}\",", …)` and `pretty_ignore_block`). Copy keys write those strings through `arboard` (or equivalent). Do not scrape widget text (avoids borders/chrome).

**Alternatives considered:** OSC 52 only (spotty on Windows); require mouse select in alt screen (poor UX).

### 6. Error modal then restore

**Choice:** Worker sends `Error`; UI shows modal; on dismiss/quit, disable raw mode / leave alt screen, then exit non-zero. Stdout mode keeps today’s `main` `Result` error path.

**Alternatives considered:** Immediately tear down TUI and print to stderr only (loses context mid-run; still must restore terminal).

### 7. Dependencies

**Choice:** `ratatui` + `crossterm` for UI; `arboard` for clipboard; `is-terminal` or `std::io::IsTerminal` for TTY detection.

## Risks / Trade-offs

- **[Risk] Alt screen + no scrollback** → Mitigation: clipboard keys for paste targets; `--raw` for classic scrollback; always block-until-`q` so users can copy before exit.
- **[Risk] Wide refactor of emitters** → Mitigation: land `Reporter` + `StdoutReporter` first with behavior-identical tests on `--raw`/captured stdout; add TUI backend second.
- **[Risk] Clipboard failures (headless, permissions)** → Mitigation: show a brief status/error on the Done screen; do not crash the run after successful work.
- **[Risk] Threading + yarn child processes** → Mitigation: keep subprocess calls on the worker only; UI thread never owns yarn; document that Ctrl+C handling may need explicit restore (implement signal-safe teardown where practical).
- **[Risk] Spec drift vs “print to stdout” tests** → Mitigation: run existing stdout assertions under `--raw` or non-TTY; add mode-selection tests for interactive path.

## Migration Plan

1. Introduce `Reporter` / events; route all user-facing emissions; default backend = stdout (no behavior change).
2. Add `--raw` and TTY detection wiring (interactive path can still call stdout backend until TUI lands).
3. Implement compact `check` TUI + Done (quit + copy) as the smaller interactive surface.
4. Implement live `auto` dashboard on the same state/Done model.
5. Error modal + terminal restore hardening.
6. Update help text for `--raw`; keep CI/scripts on non-TTY or `--raw`.

Rollback: `--raw` or non-TTY restores prior UX; removing the feature flag path is reverting the TUI backend while keeping Reporter if desired.

## Open Questions

- Exact key labels beyond `q` / resolutions / ignores (e.g. `r` and `i`) — pick at implement time; specs only require the behaviors.
- Whether “fixes” (within-range applied suggestions) share the resolutions clipboard key or a separate key — default: resolutions key copies only `suggested resolutions` body lines as today; within-range `fixes` remain display-only unless we later extend copy (not required for v1).
