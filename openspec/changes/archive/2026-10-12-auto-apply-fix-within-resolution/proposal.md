# Proposal

## Why

When `package.json` already has a resolution for a package and a newer safe version still satisfies that resolution’s range, `auto` leaves the within-range update unapplied. The final report then still lists that package under `fixes` (TUI TASKS). Within-range remediable work must be exhausted by `auto`; leftover fixes mean the run failed its job.

## What Changes

- During `auto`, treat a safe version that satisfies an existing `package.json` resolution range as a within-range fix and apply it (reset / re-resolve), instead of leaving it for the final `check` report.
- After hygiene (dropped resolutions / ignore hygiene + refresh), re-enter the within-range fix loop when the tree changed so newly unlocked within-range fixes are applied before the final `check`.
- Classification must prefer a within-range fix over a suggested outside-range resolution for the same package key, and evaluate `get_fix` against the lock entry’s own tree version (not always `tree_versions().last()`).
- Invariant: after a successful `auto`, the final report’s `fixes` section MUST be empty.

## Capabilities

### New Capabilities

- _(none)_

### Modified Capabilities

- `partial-within-range-reset`: Require applying within-range updates even when an existing resolution’s range already admits the safe version; require `auto` to finish with no remaining within-range `fixes`.
- `auto-resolutions-cleanup`: After resolution/ignore hygiene refresh, re-enter within-range fixing when the refresh can unlock further within-range work (supersedes the old non-goal of never re-auditing after drops).

## Impact

- `src/gnarl.rs` (`auto` loop / hygiene sequencing, `fix`, `classify`, `get_fix` call sites)
- Possibly `src/yarn.rs` / `src/project.rs` if a resolution pin must be widened when an exact pin blocks a within-range safe version that still fits the intended caret range
- TUI TASKS pane empties the `fixes` section after `auto` when only resolutions/ignores/unresolved remain
- Unit tests for classify preference, resolution-range within-range apply, and post-hygiene re-loop
