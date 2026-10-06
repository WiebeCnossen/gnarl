# Proposal

## Why

Yarn repos that want unattended `auto` already have a bot command and policy exit codes, but nothing in this repository shows how to wire that into Azure DevOps on a nightly schedule, including download, cache, and push-versus-PR. Teams otherwise invent conflicting wrappers; gnarl still should not grow a hosting client.

## What Changes

- Add a copy-paste Azure Pipelines nightly example (YAML plus any small helper script) that: takes the yarn working directory as a parameter, installs Yarn first, downloads a pinned gnarl release, caches that binary, runs `gnarl auto --raw --auto-ignore --install-on-change`, and commits dirty `package.json` / `yarn.lock` / `.yarnrc.yml` either to the default branch or as a PR depending on a bot `prFrom` severity setting.
- Point the README at that example from the existing unattended/bot guidance. No CLI, exit-code, or dist-target changes.

## Capabilities

### New Capabilities

None. This is documentation and a sample pipeline, not product behavior. Specs stay unchanged (`skip_specs: true`).

### Modified Capabilities

None. `auto-ignore` and `install-on-change` already define policy exits and the bot command; this change only shows how a host applies them.

## Impact

- New files under something like `examples/azure-pipelines/`.
- README link and short Azure nightly notes (parameters, Windows agent while releases are Windows-only, permissions, schedule).
- No Rust, Cargo, or GitHub release workflow changes. Consumer Azure org permissions stay the consumer's problem; the example documents them.
