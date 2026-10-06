# Proposal

## Why

`auto` always starts with `yarn install` and `yarn dedupe`, which is the right default after local dependency edits but wastes CI time when the tree is already installed. `-x` is undocumented, has no long form, and does the opposite of that CI need: it still runs the opening install+dedupe, then skips later refreshes and skips applying within-range lockfile resets.

## What Changes

- **BREAKING:** Redefine `-x` from “no further installs / do not apply auto lockfile resets / do not chain `reset` into `auto`” to **install-on-change**: skip the speculative opening `install`+`dedupe` of `auto`; still run them after this run mutates `package.json` or `yarn.lock`.
- Accept the long form `--install-on-change` as an alias of `-x`. Do not add `--no-install` (that name is used only in older specs; it is not a CLI flag today).
- Document `-x` / `--install-on-change` in `gnarl help` and the README. README bot best practice is `gnarl auto --raw --auto-ignore --install-on-change` (after the job’s own `yarn install`).
- Leave default `auto` unchanged: always open with install+dedupe so interactive use after dependency edits still refreshes the tree.
- Treat “changed” as mutations **in this process** (within-range lockfile reset, unused-resolutions drop, ignore-hygiene package reset, standalone `reset` that dirties the lockfile). Do not inspect git or mtimes for edits made before gnarl started.
- Yarnrc-only writes (including `--auto-ignore`) still MUST NOT by themselves trigger install+dedupe.
- `gnarl reset … -x` MUST still apply the lockfile reset, then chain into `auto`, and that chained auto MUST refresh (install+dedupe) before audit because the lockfile is already dirty.
- There is no remaining flag that disables all Yarn install/dedupe.

## Capabilities

### New Capabilities

- `install-on-change`: CLI `-x` / `--install-on-change` on `auto` (including default auto) and `reset`; skip auto’s opening install+dedupe unless this run already dirtied `package.json` or `yarn.lock`; still refresh after those mutations.

### Modified Capabilities

- `auto-resolutions-cleanup`: Dropping unused `package.json` resolutions still refreshes with install+dedupe. Remove the “installs disabled / `--no-install`” skip; `-x` MUST still refresh after that `package.json` write.
- `npm-audit-ignore-advisories`: Ignore-hygiene that resets packages still refreshes with install+dedupe. Remove the “`--no-install` disables follow-up install” path; `-x` MUST still refresh after those lockfile resets. Orphan-only yarnrc cleanup still does not install.

## Impact

- CLI parsing (`Options`, help, README): `-x`, `--install-on-change`; help names both spellings; README states bot best practice `gnarl auto --raw --auto-ignore --install-on-change`.
- `Gnarl::auto` loop: leading install+dedupe gated; loop continues and refreshes after lockfile resets even with `-x`.
- `Gnarl::reset` + `main` chain: `-x` no longer suppresses chaining or the post-reset refresh.
- Specs/docs that mention `--no-install` as if it were the CLI.
- CI/bots: `yarn install` then `gnarl auto --raw --auto-ignore --install-on-change`.
