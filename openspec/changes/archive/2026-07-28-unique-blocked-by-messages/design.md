## Context

`gnarl auto` runs an outer loop: install → audit → `fix` advisories → optional lockfile resets → repeat while dirty. In `fix()`, when a non-npm dependent blocks a within-range update, gnarl prints:

```text
{package} blocked by {other-package}@{version}
```

After partial within-range resets, the outer loop often continues while the same blocked ranges remain. Those lines reprint on later iterations even though fix behavior is unchanged.

`Gnarl` already keeps a process-lifetime `reset: HashSet<String>` across outer iterations. The per-iteration `done` set only prevents reprocessing the same advisory within one pass.

## Goals / Non-Goals

**Goals:**

- Print each distinct `{package} blocked by {other-package}@{version}` message at most once per program run when the same situation is encountered again on a later `auto` iteration
- Leave escalation, reset, and other logging behavior unchanged

**Non-Goals:**

- Deduplicating identical lines produced by multiple dependents within a single `fix()` call
- Deduplicating other info messages (`has no fix`, resolution forced/capped/expanded, etc.)
- Changing when packages reset or when parent advisories are enqueued

## Decisions

### Dedup at the print site with a process-lifetime set

Track previously printed blocked-by keys on `Gnarl` (alongside `reset`), and only call `out_info!` when insert succeeds.

**Alternatives considered:**

- Carry advisory `done` across outer iterations — would change processing, not just logging; out of scope
- Collect messages and print uniquely at end of run — changes output timing/ordering relative to other logs
- Dedup within `fix()` as well — user explicitly scoped this to across iterations only

### Uniqueness key = formatted message (or equivalent triple)

Key on `(package, other-package, version)` matching the printed fields (`root_name`/`module_name`, `module_name`, `tree_version`). That is enough to suppress cross-iteration repeats of the same line.

### Scope: this message only

Other repeating info lines are left alone until there is a separate ask.

## Risks / Trade-offs

- [Risk] A later iteration could have a meaningfully different situation that happens to format the same → Mitigation: key includes package, blocker package, and version; if the tree version changes, a new line still prints
- [Risk] Within-pass duplicates remain → Accepted; explicit non-goal
- [Trade-off] Slightly more state on `Gnarl` → Negligible; same pattern as `reset`

## Migration Plan

No migration. Behavior is output-only; no lockfile or config format change.

## Open Questions

None — scope confirmed: cross-iteration dedupe only for this message.
