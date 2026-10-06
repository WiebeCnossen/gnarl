# Proposal

## Why

On the interactive Done screen, suggested ignores and outside-range resolutions are copy-paste only (`[i]` / `[r]`). Applying them still means leaving the TUI, editing `.yarnrc.yml` or `package.json` by hand, and re-running gnarl. Operators already have `--auto-ignore` for unattended ignore writes; they need the same apply actions as explicit keys, plus a way to pin suggested resolutions and continue into a real `auto` refresh.

## What Changes

- On the interactive Done screen of `auto` and `check`, add **`I`** (apply all current suggested-ignore IDs to `.yarnrc.yml`) and **`R`** (write all current suggested resolutions into `package.json` `resolutions`).
- Keep lowercase **`i`** / **`r`** as clipboard copy. Empty sections remain no-op. No confirmation dialog.
- **`I`** reuses the same candidate set and yarnrc merge as `--auto-ignore`, MUST NOT trigger install/dedupe by itself, MUST NOT set policy exits 10–14, and is valid on `check` (the CLI flag stays auto-only).
- **`R`** writes structured package keys and `^version` values matching the suggestion lines (not parsed clipboard text), then runs the **full `auto` loop** with opening refresh (`refresh_first = true`) even when `-x` / `--install-on-change` was set, because this process just mutated `package.json`.
- After **`R`**, the TUI switches to the **live `auto` dashboard** (including when the original verb was `check`), then a new Done screen until `q`.
- After **`I`**, re-run **`check`** only (compact view if the session started as `check`, unless a later **`R`** already switched to live `auto`).
- `--raw` / non-TTY stays paste-only; no new CLI flags.

## Capabilities

### New Capabilities

- `apply-suggested-resolutions`: Persist the current outside-range suggested resolutions into `package.json` and continue with a full `auto` refresh (install+dedupe first because the tree is dirty).

### Modified Capabilities

- `tty-dashboard`: Done-screen keys `I` / `R` to apply; hints distinguish copy vs apply; after `R` use the live `auto` dashboard; after `I` refresh the report via `check`; interactive `check` may mutate yarnrc/package.json through these keys.
- `auto-ignore`: Interactive `I` is an exception to “without the flag, never write ignores”; it MUST NOT use policy exits 10–14.

## Impact

- TUI event loop: after the first worker finishes, accept apply keys and spawn further worker runs without tearing down the alternate screen.
- `Gnarl` / `Project`: enable `set_resolution` writes from the classified suggestion map; reuse `persist_suggested_ignores` (or equivalent) without policy status.
- `RunStatus`: `I` does not raise 10–14; a subsequent `--auto-ignore` `auto` after `R` still applies existing policy rules for that `auto` invocation.
- Tests: key no-ops, ignore merge without install, resolution writes then `auto(true)`, UI verb/layout after `R` from `check`.
- README/help: document Done-screen `I` / `R` (TTY only).
