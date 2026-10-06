# About

**gnarl** - the yarn v2/v3/v4 companion tool (Rust implementation).

This is a complete and incompatible rewrite of the Go version.

*Note*: it is highly recommended to use `yarn` version `4.15` or later with `npmMinimalAgeGate: 1d` in the `.yarnrc.yml` file in combination with Aikido safe-chain.

# Usage

```
gnarl [check | reset <packages> | auto] [-s <severity>] [--raw] [--auto-ignore] [-x|--install-on-change]
```

## Auto

This is the default operation. It will do

1. `install`
2. `dedupe`
3. `audit`
4. `reset` packages that can be fixed within the specified range
5. restart from 1 if `yarn.lock` was modified in this iteration
6. drop unused resolutions from `package.json`
7. drop orphan `npmAuditIgnoreAdvisories` entries and entries superseded by a within-range fix (resetting those packages)
8. if resolutions were removed or ignore hygiene reset packages, run `install` + `dedupe` once more
9. with `--auto-ignore`, merge suggested ignore IDs into `.yarnrc.yml` (does not by itself trigger install)
10. run `check` (including ignore overview and suggested ignores)

```
gnarl [-s <severity>]
gnarl auto --raw --auto-ignore --install-on-change
```

`--auto-ignore` is valid only on `auto` (including default `auto`). Advisory-driven reset and applied-ignore messages include severity. Within-range resets and deprecations do not affect the policy exit. A bot chooses pull/merge request vs direct push from that code; gnarl has no hosting client.

Without `--auto-ignore`, suggested ignores stay paste-only. `--raw` only forces tagged stdout (no TUI), independent of `--auto-ignore`.

`-x` / `--install-on-change` skips the opening `install`+`dedupe` of `auto`. Install+dedupe still run after this process mutates `package.json` (unused resolutions) or `yarn.lock` (within-range reset, ignore-hygiene package reset, or a chained `reset`). Yarnrc-only writes (including `--auto-ignore`) do not trigger a refresh. Best practice for bots, after the job’s own `yarn install`, is `gnarl auto --raw --auto-ignore --install-on-change`. Interactive use after local dependency edits should omit the flag so `auto` still opens with install+dedupe.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success: no new ignores written, or the run did not use `--auto-ignore` (`check`, `reset`, `help`, `info`, and plain `auto`) |
| 1 | Tool error (unknown verb, invalid flag combination, yarn/network/parse failure, and so on) |
| 10 | `auto --auto-ignore`: highest severity among **newly written** ignore IDs is `info` |
| 11 | `auto --auto-ignore`: highest newly written ignore is `low` |
| 12 | `auto --auto-ignore`: highest newly written ignore is `moderate` |
| 13 | `auto --auto-ignore`: highest newly written ignore is `high` |
| 14 | `auto --auto-ignore`: highest newly written ignore is `critical` |

Codes 10–14 are used only after a successful `auto --auto-ignore` that wrote at least one new ID. A tool error always exits `1`, even if ignores were already written.

## Check

Only runs an audit and checks what issues and fixes are available. It also:

- prints an overview of current `.yarnrc.yml` `npmAuditIgnoreAdvisories` (ID, severity, package when known)
- prints `suggested resolutions` (outside-range) and unresolved issues without inline ignore annotations
- when there are new ignore candidates (from suggested resolutions and unresolved issues, excluding IDs already in yarnrc), prints a `suggested ignores` section with the same enrichment form as the overview, followed by a paste-ready `npmAuditIgnoreAdvisories` YAML block of **new IDs only** (merge into `.yarnrc.yml` yourself)

`check` does not modify `.yarnrc.yml`.

```
gnarl check [-s severity]
```

## Reset

Removes the resolutions for a package, so that a subsequent `yarn install` will update the package.

```
gnarl reset package-names...
```

## Help

Prints version and help.

```
gnarl help
```

# Compilation

```
cargo build --release
```

The binary will be in `target/release/gnarl` (or `target/release/gnarl.exe` on Windows).
