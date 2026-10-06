# Design

## Context

See proposal.md for motivation and specs for contracts. Today `tui::run_interactive` spawns one worker (`Gnarl::auto` / `check`) and the UI thread only copies clipboard after `worker_finished`. Suggested resolutions exist as formatted section strings; `Project::set_resolution` is unused. `--auto-ignore` persist lives in `persist_suggested_ignores` and returns policy `RunStatus`. `auto(refresh_first)` already encodes “this process already dirtied the tree” via `should_open_with_refresh`.

## Goals / Non-Goals

**Goals:**

- Keep the alternate screen up across multiple worker runs.
- Apply from the same structured suggestion set the Done screen displayed.
- Reuse yarnrc merge and `auto(true)` rather than a parallel pipeline.
- Preserve `--auto-ignore` policy across a later `R` continuation that writes no new IDs.

**Non-Goals:**

- Apply actions in `--raw` / non-TTY.
- Per-item apply, confirmation dialogs, or a CLI `--apply-resolutions` flag.
- Re-parsing `format_resolution_line` / YAML as the source of truth.
- Changing Caps Lock / keyboard layout behavior beyond matching `KeyCode::Char('I'|'R')`.

## Decisions

### 1. Sequential workers on one TUI session

**Choice:** After the first worker finishes, `run_interactive` keeps the terminal and channel. `I` / `R` spawn a new worker on the same `Sender<UiEvent>` with cloned `Options`. Join each worker when it finishes; ignore `I`/`R`/`q` apply while a worker is running (`q` still quits only when `done` or error). On `R`, set `UiState.verb = Verb::Auto` before the continuation so `is_live_auto()` is true. On `I`, leave the verb unchanged. Clear `done` / `report_ready` and report sections for the new run; keep activity/fixes history for live `auto`.

**Alternatives considered:** Tear down TUI and re-enter (loses session, flashes terminal); one long-lived `Gnarl` on a command channel (heavier sharing vs today’s per-run `Gnarl::with_reporter`).

### 2. Structured apply payload on the reporter

**Choice:** When emitting the report, also send structured entries (resolution key + version; ignore IDs) alongside existing `Section` / `IgnoreYaml` events. `UiState` stores those maps. Apply workers receive a clone of that snapshot (or the IDs/keys) so writes match what the user saw. Do not parse clipboard strings. Stdout reporter ignores the structured events.

**Alternatives considered:** Re-classify on keypress (can diverge from the on-screen list); parse `"pkg@^x": "^y",` (fragile, forbidden by spec).

### 3. Split persist from policy status

**Choice:** Factor yarnrc merge into a helper that returns whether IDs were written and the max new severity. `--auto-ignore` maps that to `RunStatus`. `I` calls the helper then `check()`, and MUST NOT fold severity into process status.

**Alternatives considered:** Call `persist_suggested_ignores` from `I` and drop its `RunStatus` (easy to leak 10–14 if someone threads it into `finish_interactive`).

### 4. Resolution write then `auto(true)`

**Choice:** Apply worker: `set_resolution` for each snapshot entry (value `"^{version}"` matching `format_resolution_line`), `Project::save`, then `Gnarl::auto(true)` with the original `Options` (so `-x` still skips only a *later* speculative open; this continuation always refreshes). Original `check` does not set `auto_ignore`.

**Alternatives considered:** `auto(false)` and rely on install-on-change dirty detection inside a new `Gnarl` (a fresh instance has no dirty flag → `-x` would skip install).

### 5. Session policy is max of `--auto-ignore` persists only

**Choice:** Track `RunStatus` for the session: start from the first worker; `I` never updates it; each continuation `auto` replaces-or-maxes only the policy component from `--auto-ignore` persists (max severity among IDs newly written in this process). If a later `--auto-ignore` `auto` writes no new IDs, keep the earlier 10–14.

**Alternatives considered:** Last worker wins (would drop 14 to 0 after `R` when ignores were already written).

## Risks / Trade-offs

- **[Risk] Caps Lock turns copy into apply** → Mitigation: hints label `[r]` copy vs `[R]` apply; no extra confirm (per spec).
- **[Risk] Stale snapshot vs disk if the user edits files while Done** → Mitigation: apply the displayed snapshot (spec: what Done showed); next report re-audits.
- **[Risk] Nested workers vs yarn children** → Mitigation: only one worker at a time; UI thread never calls yarn.
- **[Risk] `package.json` pretty-rewrite churn** → Mitigation: existing `Project::save` behavior; accept same as other resolution edits.
- **[Risk] Policy max rules surprise bots** → Mitigation: bots use `--raw` and never hit `I`/`R`; document TTY-only.

## Migration Plan

Additive TUI behavior. No flag or stdout change. Ship behind existing interactive mode; `--raw` unchanged. Rollback is revert.

## Open Questions

None that affect specs or this approach.
