# Design

## Context

See proposal.md for motivation. README already documents `gnarl auto --raw --auto-ignore --install-on-change` and exit 0 / 1 / 10–14; gnarl has no hosting client. `dist-workspace.toml` currently ships only `x86_64-pc-windows-msvc`. There is no `examples/` tree yet. Policy exits 10–14 fire only when `--auto-ignore` writes **new** IDs; a dirty tree on exit 0 (within-range lockfile, unused resolutions, ignore hygiene) is still a successful auto-fix.

## Goals / Non-Goals

**Goals:**

- A drop-in Azure Pipelines sample a Yarn repo can copy, scheduled nightly on the default branch.
- Host-side policy: `prFrom` maps max new-ignore severity (exit 10–14) to push vs PR; exit 0 with a dirty tree still commits (push unless `prFrom` is not used that way—exit 0 is always below every severity threshold).

**Non-Goals:**

- Azure, git, or PR code inside gnarl.
- Extra cargo-dist targets (Linux) or GitHub Actions sample in this change.
- Running this pipeline against the gnarl repo itself.
- An `always` PR mode for lockfile-only diffs.

## Decisions

### 1. Copy-paste example, not a template repo contract

**Choice:** Shared helper source in `examples/nightly/Invoke-GnarlNightly.ps1`; consumer Yarn repos copy it to `scripts/Invoke-GnarlNightly.ps1`. Host YAML in `examples/azure-pipelines/azure-gnarl-pipeline.yml` (a second ADO pipeline, not a replacement for the repo's main CI YAML) and `examples/gitlab-ci/gnarl.gitlab-ci.yml` default to that consumer path. README links here from the existing bot paragraph.

**Why:** Consumers live on Azure DevOps; this repo is GitHub. A sample they copy (or vendor as a template) does not require a shared ADO template repo. YAML owns schedule, checkout, cache, Node/Yarn, and Azure CLI; the script owns `$LASTEXITCODE`, `prFrom`, git add/commit, and PR-vs-push so 10–14 never fail the `PowerShell@2` task by accident.

**Alternatives:** README-only prose (easy to drift); a reusable template in another repo (better for many internal consumers, not something this GitHub project can host).

### 2. Schedule isolation

**Choice:** `trigger: none`, `pr: none`, `schedules` cron (e.g. `0 2 * * *`) on the default-branch parameter, `always: true`.

**Why:** Nightly audit must run even with no commits. CI-on-push would recurse when the bot pushes. PR builds must not commit into contributor branches.

**Alternatives:** Path filters (still retrigger other pipelines; this job would still re-enter unless trigger is off).

### 3. Parameters

| Parameter | Default | Role |
|-----------|---------|------|
| `yarnWorkingDirectory` | `.` | Directory that already contains `yarn.lock` (not discovered) |
| `gnarlVersion` | a released version string, no `latest` | Cache key and download URL |
| `prFrom` | `high` | `none` / `info` / `low` / `moderate` / `high` / `critical` |
| `defaultBranch` | `main` | Push target and PR target |

`prFrom: none` means never open a PR (push all dirty successes). Exit `1` always fails the job. Clean tree is a no-op. Dirty + exit `0` or exit code `<` threshold of `prFrom` → commit and push `defaultBranch`. Dirty + exit `>=` threshold → branch `gnarl/audit-ignores` (fixed name), push, `az repos pr create` or reuse the open PR.

**Why severity names:** operators should not memorize 10–14. Mapping: info=10 … critical=14.

**Alternatives:** Threshold as raw exit code; always-PR even on exit 0 (deferred).

### 4. Agent and download

**Choice:** `vmImage: windows-latest`. Each job downloads the pinned cargo-dist zip (`x86_64-pc-windows-msvc`) from GitHub Releases into a temp directory and extracts `gnarl.exe` there. No pipeline cache: the archive is small and Cache@2 added more failure modes than time saved.

**Why:** Matches current dist targets. Pin the version; do not use `latest`. Implementation uses the real asset/tag naming from a current release (`vX.Y.Z` / `gnarl-x86_64-pc-windows-msvc.zip`).

**Alternatives:** Azure Artifacts feed; `Cache@2` (rejected: miss + missing directory failed the post-job).

### 5. Yarn before gnarl

**Choice:** Node + corepack in `yarnWorkingDirectory`, then `yarn install`, then gnarl with `--install-on-change`.

**Why:** Matches README bot best practice. The example can use `UseNode@1` and `corepack enable`; consumers adjust Node/Yarn to their `packageManager` field.

### 6. Git / identity

**Choice:** `checkout: self` with `persistCredentials: true`; detached HEAD → checkout `defaultBranch`; `git add` only `package.json`, `yarn.lock`, `.yarnrc.yml` under `yarnWorkingDirectory`; commit as a bot identity; push with `System.AccessToken`. PR via Azure CLI and the same token (`SYSTEM_ACCESSTOKEN`). Header comments list required permission bits (Contribute, Create branch, Contribute to pull requests) and that required reviewers block direct push unless the identity may bypass or `prFrom` is set so everything becomes a PR.

**Why:** ADO’s usual job-identity path; scoped add avoids `node_modules`. Idempotent `gnarl/audit-ignores` avoids a PR per night.

**Alternatives:** PAT in a secret (works, more to rotate); new branch name per run (PR spam).

## Risks / Trade-offs

- **[Risk] Branch policies reject push to `main`** → Document bypass vs raising `prFrom` / using `none` only when bypass exists; sample cannot grant org permissions.
- **[Risk] GitHub rate limit or missing Linux binary** → Pin the version; Windows-only sample until dist grows targets.
- **[Risk] Exit 10–14 fails the Azure step** → Helper captures `$LASTEXITCODE` and maps 1 to failure itself.
- **[Risk] Recursion via other pipelines** → This sample has `trigger: none`; other CI on the default branch is intended to run after a bot push.

## Migration Plan

Docs-only for gnarl: merge example + README link. Consumers copy YAML/script into an Azure repo, set permissions, pin `gnarlVersion`. Rollback is delete the copied pipeline. No gnarl release coupling beyond “use a published Windows zip”.
