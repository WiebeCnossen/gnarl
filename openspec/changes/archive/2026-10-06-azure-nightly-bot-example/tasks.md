# Tasks

## 1. Nightly helper

- [x] 1.1 Add `examples/azure-pipelines/Invoke-GnarlNightly.ps1` with `prFrom` (`none`/`info`/`low`/`moderate`/`high`/`critical` → 10–14), treat exit `1` as failure, exit `0` as below every PR threshold, scoped `git add` of `package.json`/`yarn.lock`/`.yarnrc.yml` under `yarnWorkingDirectory`, `[skip ci]` commit, push to `defaultBranch` vs `gnarl/audit-ignores` + `az repos pr create`; verify a Pester (or equivalent) test file next to it covers threshold matrix including `none` and dirty exit `0`
- [x] 1.2 In that helper (or a sibling script the YAML calls), download the pinned `x86_64-pc-windows-msvc` GitHub Release zip into a temp dir and place `gnarl.exe` in the cache directory only after a successful extract, using the real cargo-dist tag/asset name from a current release; verify comments or parameters document the URL shape and a dry-run path refuses to write the cache dir when extract would fail

## 2. Pipeline YAML

- [x] 2.1 Add `examples/azure-pipelines/azure-pipelines.yml`: `windows-2022`, `trigger: none`, `pr: none`, nightly cron with `always: true`, parameters `yarnWorkingDirectory` / `gnarlVersion` (no `latest`) / `prFrom` default `high` / `defaultBranch`, `Cache@2` key `gnarl | windows | $(gnarlVersion)`, checkout with `persistCredentials`, Node + corepack + `yarn install` in the working directory, then the helper with `gnarl auto --raw --auto-ignore --install-on-change`; verify the YAML contains those knobs and header comments for Build Service permissions and branch-policy limits

## 3. README

- [x] 3.1 From the existing unattended/bot paragraph, link to `examples/azure-pipelines/` and note nightly schedule, Windows-only zip, cache, and `prFrom`; verify README still contains `gnarl auto --raw --auto-ignore --install-on-change` and that any existing help/README tests that scan the bot command still pass

## Workflow follow-up

- Archive the change after review (`/opsx-archive`).
