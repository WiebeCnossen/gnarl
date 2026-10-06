# Proposal

## Why

A repo-maintenance bot can already run `auto` for within-range lockfile resets, but suggested ignores stay paste-only and a successful run always exits 0. The bot therefore cannot apply the remaining audit IDs or tell from the process status whether a commit is a safe push or needs a pull/merge request.

## What Changes

- Add `--auto-ignore` as an explicit opt-in, independent of `--raw`. `--raw` stays a presentation switch (tagged stdout even on a TTY).
- When `auto` runs with `--auto-ignore`, after the existing reset loop and ignore hygiene, gnarl writes the same suggested-ignore candidate IDs it would otherwise only print into `.yarnrc.yml` `npmAuditIgnoreAdvisories` (merge with existing entries).
- Successful `auto --auto-ignore` exits **10–14** according to the **maximum severity of IDs newly written this run**, in every UI mode (tagged stdout and interactive TUI after quit). Exit **0** when no new ignores were written. Tool/crash failures stay **1** and take precedence.
- Within-range resets and deprecations never affect that policy exit. The PR-vs-push threshold stays in the bot.
- Reset-fix and auto-applied-ignore messages include the severity of the advisory being handled.
- `--auto-ignore` is invalid on verbs other than `auto`. Without the flag, `auto` does not write suggested ignores and still exits 0 on success.

## Capabilities

### New Capabilities

- `auto-ignore`: Opt-in `auto` behavior to apply suggested ignore candidates, report severity on those writes and on advisory-driven resets, and encode the max newly written ignore severity as process exit status 10–14 (all modes).

### Modified Capabilities

- `npm-audit-ignore-advisories`: Candidate set, yarnrc write rules (merge, integer IDs, line endings), and the final `check` after `auto` remain the source of truth; extend so `--auto-ignore` persists those candidates before the final report so they appear as current ignores rather than suggestions.
- `tty-dashboard`: `--auto-ignore` MUST NOT change UI mode selection. After a successful interactive Done + quit, the process MUST use the same policy exit as tagged stdout. Applied-ignore and reset-fix lines shown in the live fixes region MUST include severity when this change requires it.

## Impact

- CLI parsing (`Options`, help): `--auto-ignore`.
- Process lifetime (`main`, TUI worker join): success is no longer always exit 0.
- `auto` pipeline in `gnarl`: apply suggested ignores after hygiene, before final `check`.
- Yarnrc writers: merge new IDs using existing splice/line-ending behavior.
- Fix event text: lockfile resets motivated by advisories, and newly written ignores.
- Bots: `gnarl auto --raw --auto-ignore`; interpret 10–14 themselves. No Git hosting client in gnarl.
