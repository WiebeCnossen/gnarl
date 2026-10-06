# Tasks

## 1. CLI

- [x] 1.1 Rename `Options.no_install` to `install_on_change` and parse both `-x` and `--install-on-change`; verify unit tests that default auto is unset, `-x` and `--install-on-change` set it on `auto`/`reset`, and `check -x` still parses
- [x] 1.2 Document `-x` / `--install-on-change` in `gnarl help` (both spellings, skip opening install+dedupe, still refresh after this-run `package.json`/`yarn.lock` mutations) and README (same meaning plus bot best practice `gnarl auto --raw --auto-ignore --install-on-change`); verify help names both spellings and README contains that bot command

## 2. Refresh policy and auto loop

- [x] 2.1 Add a small helper for opening refresh (`install_on_change`, `refresh_first`) and give `Gnarl::auto` a `refresh_first` argument; verify unit tests: default auto refreshes, `-x` without `refresh_first` skips, `-x` with `refresh_first` refreshes
- [x] 2.2 Gate the top-of-loop install+dedupe on that helper; on within-range lockfile reset set `refresh = true` and continue instead of breaking on `-x`; verify a test or assertion that `-x` still applies the reset and schedules the next-iteration install+dedupe
- [x] 2.3 Always run post-hygiene install+dedupe when unused resolutions were dropped or ignore hygiene reset packages (drop the old `!no_install` guard); verify `-x` still refreshes in those cases and does not refresh for yarnrc-only / `--auto-ignore` writes

## 3. Reset chain

- [x] 3.1 Make `reset` return lockfile-dirty only (no `install_on_change` suppression) and pass `refresh_first = true` into tagged-stdout and TUI `auto` when chaining; verify dirty reset `-x` chains and would open auto with install, and a no-op reset does not chain
- [x] 3.2 Keep default `auto` / TUI auto at `refresh_first = false`; verify interactive and tagged `gnarl auto` without `-x` still open with install+dedupe

## 4. Clippy

- [x] 4.1 Run `cargo clippy --all-targets` and clear every warning
