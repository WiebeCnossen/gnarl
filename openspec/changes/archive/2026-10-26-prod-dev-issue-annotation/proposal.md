# Proposal

## Why

Operators triage audit findings without knowing whether a package is reachable only through `devDependencies` or also through production `dependencies`. That distinction changes urgency, but gnarl’s report lines today do not show it.

## What Changes

- Annotate each `unresolved issues` body line with `prod` or `dev-only`
- Annotate each `suggested ignores` enrichment line with the same labels
- Derive labels by comparing a full recursive audit to a production-environment recursive audit (`yarn npm audit --environment production`)
- Do **not** change classification, filters, exit codes, auto-ignore policy, paste-ready ignore YAML, fixes/resolutions, hits, or the current-ignore overview

## Capabilities

### New Capabilities

- `prod-dev-annotation`: Classify advisories as production-reachable vs:dev-only and surface that label on unresolved-issue and suggested-ignore enrichment lines

### Modified Capabilities

- `npm-audit-ignore-advisories`: Extend suggested-ignores enrichment lines to include the prod/dev-only label (YAML block stays IDs-only)

## Impact

- Audit path: second `yarn npm audit --json --recursive --environment production` (or equivalent) when building the final report
- Report formatting in `gnarl` check/report emission for `unresolved issues` and `suggested ignores`
- Interactive Done screen inherits the same section body strings (no separate TUI format)
- No CLI flags, exit-code, or yarnrc write-behavior changes
