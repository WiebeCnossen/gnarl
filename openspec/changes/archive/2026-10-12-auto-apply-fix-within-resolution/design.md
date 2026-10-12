# Design

## Context

See proposal.md for motivation. Today `auto` loops audit → within-range `fix` → lockfile reset → install until clean, then hygiene (`reset_resolutions`, ignore drop) with at most one refresh, then `check`. `classify` builds the `fixes` / `suggested resolutions` report; `fixes.retain(|k,_| !resolutions.contains_key(k))` prefers resolutions over fixes for the same key. `get_fix` in `classify` always uses `advisory.tree_versions().last()` rather than the lock entry’s version. Existing `package.json` resolutions are never consulted when deciding whether a within-range update can land; an exact pin can make reset+install a no-op while `self.reset` blocks retries, leaving `fixes` in the final TASKS pane.

## Goals / Non-Goals

**Goals:**

- Exhaust within-range remediable advisories before the final `check`
- Honor existing resolution ranges that already admit a safe version
- Widen exact pins that block such a safe version when the dependency request still allows it
- Fix classify so within-range wins over suggested resolution and uses the correct tree version

**Non-Goals:**

- Auto-applying outside-range suggested resolutions (still operator/`[R]` / `--auto-ignore` territory)
- Changing when a resolution is considered unused for hygiene drop heuristics beyond what is needed to apply a within-range safe version
- Changing TUI layout or copy keybindings

## Decisions

1. **Outer auto structure: fix loop → hygiene → if hygiene refreshed, fix loop again → check**  
   Prefer wrapping the existing within-range loop so hygiene can unlock another pass, over folding hygiene into every iteration (would re-run `reset_resolutions` constantly). Cap implicit progress via existing `self.reset` / dirty flags so a pin that still cannot move does not spin forever—after widening the pin once, a normal reset iteration must succeed or the advisory is no longer within-range classifiable as `fixes`.

2. **Within-range from lock descriptors (post-resolution), then widen pins if needed**  
   `classify` already uses lock entry requests (e.g. `handlebars@npm:^4.7.9` after `"handlebars@4.7.8": "^4.7.9"`). `fix` / ignore hygiene MUST use that same view — not only reverse-dependent declared ranges (which can stay at exact `4.7.8` and miss `4.7.10`).  
   Additionally, when applying a within-range fix, inspect matching `package.json` resolutions:
   - If the resolution range already satisfies the chosen safe version → lockfile reset only.
   - If the resolution is an exact (or otherwise blocking) pin below that safe version → update to `^{version}`, save, then reset.  
   Prefer widen over dropping the resolution entirely.

3. **Classify: correct tree version + prefer fixes**  
   Pass the lock entry’s `tree_version` into `get_fix` (the loop variable already resolved for that entry). Replace `fixes.retain(!resolutions)` with `resolutions.retain(|k,_| !fixes.contains_key(k))` so within-range wins for the same key.

4. **Postcondition enforcement**  
   Rely on the re-loop + resolution widen path rather than a hard error if `fixes` is non-empty after `check`. Add a unit/integration assertion in tests; optionally emit a debug activity if a fix remains (implementation detail—do not fail the process solely for that unless tests prove a reachable stuck state).

## Risks / Trade-offs

- [Extra audit/install cycles after hygiene] → Only when hygiene refreshed; matches the new invariant
- [Widening an exact pin surprises operators who pinned deliberately] → Only when a newer safe version is within-range for the dependency request; outside-range cases stay as suggested resolutions
- [Infinite loop if widen+reset never clears the advisory] → `self.reset` and “only widen when safe version is known” keep progress finite; leftover work must classify as resolution/unresolved, not `fixes`

## Migration Plan

No migration. Behavior change for `auto` / final `check` only; operators may see fewer leftover `fixes` and occasional resolution value updates from exact → caret safe version.
