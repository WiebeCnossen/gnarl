# Design

## Context

See `proposal.md` for why. Today `auto` loops install/dedupe/audit/within-range lockfile reset, then hygiene (`reset_resolutions`, drop orphan/superseded ignores), then `check`. Suggested ignores are print-only. `main` and the TUI worker return `Result<(), Error>` (success → process exit 0). `[FIX!] reset {package}` has no severity. `--raw` only forces tagged stdout.

Candidate membership and yarnrc splice/line endings already live in `npm-audit-ignore-advisories`; this change reuses that write path.

## Goals / Non-Goals

**Goals:**

- Thread a run outcome (policy exit 0 / 10–14) from `auto` through tagged stdout and TUI-after-`q` without treating it as `Error`.
- Share one classification of ignore candidates between “apply then check” so the final report cannot disagree with what was written.
- Pass advisory severity into lockfile reset messages without teaching `locks` about audits.

**Non-Goals:**

- Git hosting, bot thresholds, or changing `-s` semantics.
- Policy exits for `check` or for runs without `--auto-ignore`.
- Extra install/dedupe after ignore writes.

## Decisions

### 1. `--auto-ignore` is a boolean on `Options`, validated after the verb is known

Parse it next to `--raw` in `Options::read`. After `Command` has a `Verb`, if `auto_ignore` is set and the verb is not `Auto`, return `Error` (process exit 1). Default verb (empty args) is `Auto`, so `gnarl --auto-ignore` is valid. `gnarl reset … --auto-ignore` is invalid even when reset would chain into `auto`.

**Alternative:** Infer automation from `--raw` or non-TTY. Rejected in exploration; bots pass both `--raw` and `--auto-ignore`.

### 2. `Gnarl::auto` returns `Result<RunStatus, Error>` instead of `Result<(), Error>`

`RunStatus` carries `policy_exit: u8` (0 or 10–14). `check` / standalone `reset` keep success as 0. `main` maps `Err` → exit 1 (unchanged reporting) and `Ok(status)` → `std::process::exit(status.policy_exit)` (or `Termination`). Do not encode 10–14 as `Error`; that would print a failure message and collide with crash handling.

TUI: worker returns `RunStatus`; after Done + `q` and terminal restore, `run_interactive` returns that status. Error modal path still `Err`.

**Alternative:** Always `process::exit` inside `auto`. Rejected: TUI must restore the terminal first.

### 3. Classify once, write, then `check`

Extract the advisory walk that builds deprecations / within-range fixes / outside-range resolutions / unresolved / ignore suggestions (today inlined in `check`) into a shared classify step. Auto-ignore: after hygiene, classify, merge new suggestion IDs into yarnrc via existing `set_npm_audit_ignore_advisories` + `save`, emit `UiEvent::Fix` per new ID (include severity and ID; overview-like package range when known), record max `Severity` among newly written IDs, then `check()` which classifies again against the updated yarnrc (suggestions empty, overview includes the new IDs).

Policy mapping: `None` → 0; `Info` 10 … `Critical` 14. Hygiene drops and within-range resets do not update this max.

**Alternative:** Parse suggested YAML from the report. Fragile. **Alternative:** Write during `check`. Would mix reporting and mutation; spec wants write before final check.

### 4. Reset messages: severity is caller-supplied

`Locks::reset_one` today emits `reset {package}`. Extend reset to accept per-package optional severity (max of advisories that queued that package in this dirty cycle). `auto` already collects `resets: Vec<String>`; track `HashMap<String, Severity>` with max on insert. Standalone `gnarl reset` passes no severity → keep `reset {package}`.

Hygiene within-range ignore-drop resets use the dropped advisory’s severity the same way (still advisory-driven). They still do not affect policy exit.

**Alternative:** Second `Fix` event from `gnarl` in addition to locks. Duplicate lines in stdout/TUI.

### 5. Yarnrc merge helper

Add a small merge on `YarnRc` (existing IDs + new IDs, skip duplicates after string normalization) rather than replacing the sequence. Reuse `save()` so CRLF/LF and quoted IDs stay as specified.

## Risks / Trade-offs

- **[Risk] TUI user sees exit 13 after `q`** → Specified; help text documents 10–14. Without `--auto-ignore`, exit stays 0.
- **[Risk] Classify-twice drift** → Same function for apply and check; tests on candidate filtering.
- **[Risk] Scripts treat any non-zero as failure** → Only `--auto-ignore` uses 10–14; document in help/README.
- **[Trade-off] `reset --auto-ignore` rejected** → Slightly inconvenient vs chaining; keeps the flag verb-scoped as specified.

## Migration Plan

Additive flag. Existing `auto` / `check` / `reset` unchanged. README + `gnarl help`: `--auto-ignore`, exit table, bot example `gnarl auto --raw --auto-ignore`. Rollback: stop passing the flag.

## Open Questions

None; severity strings in messages must include the `Severity` label (`high`, `critical`, …) but exact punctuation may match existing `[FIX!]` style.
