# Proposal

## Why

gnarl’s user-facing output is a flat stream of tagged `println!` lines. Activity, applied fixes, resulting state, and pasteable suggestions blur together, and the tool never feels like a deliberate UI. On an interactive TTY we can present a live dashboard (and a compact report for `check`) with ratatui while keeping today’s tagged stdout for pipes, CI, and an explicit `--raw` escape hatch.

## What Changes

- Add a TTY **live dashboard** for `auto`: activity and applied fixes update during the run; KPIs, existing ignores, and next-action suggestions appear only when the final report completes.
- Add a TTY **compact report view** for `check` (thin computing status, then the same completion layout).
- On TTY UI completion, **always block until `q`**.
- Clipboard keybindings copy suggestion payloads **exactly** as today’s stdout fragments (resolutions JSON lines; `pretty_ignore_block` YAML for ignores).
- Errors in TTY UI show as a **modal**, then restore the terminal cleanly.
- Non-TTY (not a TTY) keeps **current tagged stdout** behavior and exits as today (no block-for-`q`).
- New **`--raw`** flag forces the tagged-stdout path even on a TTY.
- `reset` that does not chain into `auto` stays **stdout only**.
- `help` / `info` stay stdout.
- Introduce a presentation layer (events/reporter) so business logic no longer owns `println!` as the only channel.
- New dependencies: `ratatui`, `crossterm`, and a clipboard crate (e.g. `arboard`).

## Capabilities

### New Capabilities

- `tty-dashboard`: Interactive TTY surfaces for `auto` (live dashboard) and `check` (compact report), UI mode selection (`--raw` / non-TTY fallback), Done-screen input (quit + clipboard copy), and TUI error modal behavior.

### Modified Capabilities

- `npm-audit-ignore-advisories`: Presentation of ignore overview, suggested ignores (including paste-ready YAML), and related report sections MUST remain content-equivalent, but the delivery channel becomes stdout under raw/non-TTY and dashboard panels / clipboard under TTY UI.
- `unique-blocked-by-messages`: Per-run uniqueness of blocked-by messages MUST apply to the activity presentation channel (stdout or dashboard activity log), not only to direct `println!`.

## Impact

- CLI: `Command` / options parsing gains `--raw`; `main` chooses UI mode before `auto`/`check`.
- Core: `Gnarl`, `Yarn`, `Locks`, `Npm`, and `ux` macros move behind a reporter/event sink; stdout backend preserves current tags for compatibility.
- UX: new ratatui modules for live dashboard and compact report; clipboard integration for `[r]` / `[i]` (or equivalent keys).
- Tests: non-TTY / `--raw` paths should keep asserting tagged stdout; TUI paths need focused behavior coverage where practical (mode selection, Done gating, copy payload freestanding helpers).
- Specs above that currently say “print to stdout” need deltas so TTY UI is not a silent contradiction.
