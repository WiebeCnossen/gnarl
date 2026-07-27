## Why

After `auto`, `check` can still list within-range fixes when a package has multiple non-overlapping requested ranges: some ranges can update in-range while others cannot. Today `auto` only resets when every range is fixable (`fixable && !blocked`), so partial wins are left on the table and the final check looks unfinished.

## What Changes

- In `auto`, reset a package when **any** dependent request has a within-range fix, even if other non-overlapping ranges remain blocked
- Continue escalating blocked ranges via parent advisories (existing behavior)
- Keep ignore auto-drop strict: only drop an ignore when the advisory is fully within-range clear (no blocked sibling ranges). Partial resets MUST NOT trigger ignore drops
- Decouple ignore-drop criteria from the (now looser) normal reset predicate in the ignore-advisories spec

## Capabilities

### New Capabilities

- `partial-within-range-reset`: When `auto` finds a within-range fix for at least one requested range of a package, reset that package so the partial update applies, even if other ranges of the same package remain blocked

### Modified Capabilities

- `npm-audit-ignore-advisories`: Clarify that auto-drop of ignores for within-range fixes requires the advisory to be fully within-range remediable (no blocked ranges), not merely that a partial reset would occur

## Impact

- `src/gnarl.rs`: `fix` predicate (and possibly messaging); leave `within_range_resettable` on the strict path
- Spec wording for ignore within-range drop so it no longer says “same criteria as normal advisory reset”
- No CLI surface or dependency changes
