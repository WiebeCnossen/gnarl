# Design

## Context

See `proposal.md` for why. Today `Gnarl::auto` always `install`+`dedupe` at the start of every loop iteration. `Options.no_install` (`-x` only) then **breaks** the loop without applying lockfile resets and skips the post-hygiene refresh; `reset` returns `dirty && !no_install`, so `-x` also skips chaining into auto. Specs mention `--no-install`, which is not parsed. `--auto-ignore` already must not trigger install by itself. Reset always uses tagged stdout, then may call `run_verb_inner(Auto, …)` which constructs a **new** `Gnarl`, so “lockfile already dirtied this process” is not visible unless the caller passes it.

## Goals / Non-Goals

**Goals:**

- Gate the **opening** install+dedupe of `auto` on (not install-on-change) OR (this invocation already has a pending lockfile/package.json refresh).
- Keep one refresh path (`Yarn::install` then `Yarn::dedupe`) after in-run mutations, including with `-x`.
- Pass pending refresh across the reset→auto process boundary without git/mtime detection.

**Non-Goals:**

- A flag that disables all Yarn install/dedupe.
- Detecting edits made before the process started.
- Changing when resolutions are unused or when ignore hygiene resets packages.
- Changing `--auto-ignore` candidate/write rules.

## Decisions

### 1. Rename the option; parse both spellings

Replace `Options.no_install` with `install_on_change`. Parse `-x` and `--install-on-change` onto that field. Help documents both spellings and the skip-opening-refresh meaning. README documents the option and uses `gnarl auto --raw --auto-ignore --install-on-change` as the bot example (replacing today’s `gnarl auto --raw --auto-ignore`). `check` may still parse the flag (as today with `-x`) as a no-op.

**Alternative:** Keep the internal name `no_install`. Rejected: the semantics inverted.

### 2. `auto` takes `refresh_first: bool`; loop installs only when needed

```
refresh = refresh_first || !install_on_change
loop:
  if refresh: install; dedupe
  audit/fix
  if within-range resets:
    apply lockfile reset
    refresh = true
    continue
  break
hygiene
if package.json resolutions dropped or ignore-hygiene lockfile resets:
  install; dedupe
persist auto-ignore if requested
check
```

Default auto: `refresh_first = false` and `!install_on_change` → first iteration still installs. `auto -x`: first iteration skips install unless `refresh_first`. After a reset in the loop, `refresh = true` so the **next** iteration installs even with `-x` (same as today’s “install at top of next iteration”, minus the skip-on-`-x` break).

**Alternative:** Install immediately after reset in the same iteration, then audit again. Equivalent cost; keep top-of-loop install to stay close to the current structure.

**Alternative:** Change the default to skip opening install. Rejected in exploration (interactive use after dependency edits).

### 3. Reset chains whenever the lockfile is dirty; pass `refresh_first`

`Gnarl::reset` returns whether the lockfile changed (drop `&& !no_install`). `main` keeps chaining into auto when that is true, calling `auto`/`run_verb_inner` with `refresh_first = true` so the new `Gnarl` still refreshes before the first audit under `-x`.

TUI auto after a stdout reset uses the same `refresh_first` (reset itself is still stdout-only).

**Alternative:** Have `reset` run install+dedupe itself, then chain auto with `refresh_first = false`. Rejected: would double-install on default auto (reset install + auto opening install). Passing `refresh_first` preserves “reset never installs; the following auto does.”

### 4. Hygiene follow-up ignores the flag

Post-loop install+dedupe when `resolutions_dirty || !ignore_resets.is_empty()` always runs (no `!no_install` guard). `--auto-ignore` stays after hygiene and still does not set that condition by itself.

### 5. Tests

- Parse: `-x` and `--install-on-change` set the option; default auto does not.
- Policy helper (or equivalent) for opening refresh: `(install_on_change, refresh_first)` → skip vs run.
- Auto/reset wiring: `-x` no longer prevents lockfile reset application or reset→auto chain; hygiene still refreshes after package.json/lockfile mutations. Prefer existing unit/harness style; do not require a live Yarn CI job solely to prove the skip.

## Risks / Trade-offs

- **[Risk] Scripts used `-x` to avoid Yarn entirely** → Breaking; there is no replacement. Document in help/README.
- **[Risk] `auto -x` after local `package.json` edits audits a stale tree** → Intended; default auto still opens with install. CI contract is `yarn install` then `gnarl auto -x`.
- **[Risk] `refresh_first` forgotten on TUI chain** → Same call path as tagged stdout `run_verb_inner`.
- **[Trade-off] No “never install” flag** → Simpler CLI; matches current product (specs’ `--no-install` was never a real flag).

## Migration Plan

Ship as a CLI behavior change on `-x`. Callers that wanted the old “one opening install, then freeze” must stop passing `-x` or switch to `check` if they did not want resets. Rollback: revert the option semantics.

CI: `yarn install` (existing) then `gnarl auto --raw --auto-ignore --install-on-change`.
