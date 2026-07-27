## Context

`Gnarl::fix` walks every lock dependent of an advisory’s module. For each request range it sets `fixable` if a within-range update exists, otherwise `blocked` (and may push a parent advisory). Reset currently requires `fixable && !blocked`, so mixed packages never reset.

`check` classifies per request, so those packages still appear under “fixes” after `auto`. `Locks::reset` already removes all lock keys for a package name; install/dedupe re-resolves every range, which is enough to apply a partial win.

Ignore hygiene uses `within_range_resettable` with the same all-or-nothing predicate. That strictness must stay: a partial win does not fully clear the advisory, so the ignore ID must remain.

## Goals / Non-Goals

**Goals:**

- Reset on any within-range win during `auto`, including when sibling ranges are blocked
- Preserve parent escalation for blocked ranges in the same pass
- Keep ignore auto-drop only when every relevant range is within-range remediable
- After `auto` + install, `check` MUST NOT still list a within-range fix for ranges that were resettable

**Non-Goals:**

- Per-range lockfile surgery (reset remains whole-package)
- Changing resolution / outside-range behavior
- Dropping ignores when only a partial within-range fix exists
- New CLI flags or UX beyond optional clarity messaging

## Decisions

### 1. Loosen only the `fix` reset predicate

- **Choice:** Return “should reset” when `fixable && !self.reset.contains(module)`, ignoring `blocked`.
- **Why:** Matches “any within-range win.” Parent bubbling already runs in the dependent loop before the return, so blocked ranges still escalate.
- **Alternative:** Separate “full” vs “partial” code paths — unnecessary; same reset + bubble outcome.

### 2. Keep `within_range_resettable` strict

- **Choice:** Leave ignore-drop on `fixable && !blocked` (and not already reset).
- **Why:** Partial remediation does not resolve the advisory; auto-drop would silence a still-present ID incorrectly.
- **Alternative:** Share one helper with a `strict` flag — optional refactor; behavior split is the requirement.

### 3. Spec decoupling

- **Choice:** Update `npm-audit-ignore-advisories` so within-range ignore drop is defined as “fully within-range remediable,” not “same criteria as normal reset.”
- **Why:** After this change those criteria diverge; the old wording would force ignore drops on partial resets.

### 4. Loop safety

- **Choice:** Rely on existing `self.reset` set and the next audit after install: fixed ranges stop being within-range fixable; remaining blocked ranges no longer set `fixable` alone into a re-reset (or package is already in `self.reset`).
- **Why:** No new loop control needed for the mixed case.

## Risks / Trade-offs

- **[Risk]** Resetting a mixed package re-resolves the blocked range too, which could move it unexpectedly → **Mitigation:** Same as today’s whole-package reset; yarn constraints still apply; blocked range still lacks an in-range safe version so it should stay vulnerable until resolution/ignore.
- **[Risk]** Implementers might “fix” ignore-drop to match the new reset predicate → **Mitigation:** Explicit modified requirement + scenario that mixed packages keep the ignore.
- **[Trade-off]** Advisory may remain after partial reset — expected; remaining work shows as resolution/unresolved, not as a leftover within-range fix for the fixed range.
