# Design

## Context

See proposal.md for motivation. Today `Yarn::run_audit` runs `yarn npm audit --json --recursive` with no `--environment`. Classification builds unresolved and ignore-suggestion lines without environment info. Suggested-ignore enrichment is `"{id}  {severity}  {module}@{range}"`; unresolved lines are `"\"{module}@{request}\""`. Yarn already supports `--environment production` to exclude:devDependencies (and their transitive closure under Yarn’s audit semantics).

## Goals / Non-Goals

**Goals:**

- Attach `prod` / `dev-only` to unresolved and suggested-ignore enrichment lines using Yarn’s production audit as source of truth
- Keep one shared string shape for tagged stdout and the interactive Done screen
- Minimize churn: no new CLI flags, no policy/exit changes

**Non-Goals:**

- Graph walk over `yarn.lock` / package.json to re-derive reachability
- Filtering advisories, changing ignore candidates, or changing KPI counts
- Annotating fixes, resolutions, hits, ignore overview, or paste YAML
- Work around Yarn nested-workspace production-audit bugs beyond documenting the dependency on Yarn’s behavior

## Decisions

### 1. Dual audit set-diff (not lockfile reachability)

Run a second recursive audit with `--environment production`. An advisory ID present in both sets → `prod`; only in the full set → `dev-only`.

**Why:** Matches Yarn’s documented production semantics; avoids reimplementing workspace/dev traversal (historically buggy even inside Yarn).

**Alternative considered:** Walk `Locks::dependents` to root `dependencies` vs `devDependencies`. Rejected for annotation-only scope: more code, easy to diverge from Yarn audit membership.

### 2. Key classification by advisory ID

Compare sets of advisory IDs (normalized string form already used for ignores), not package names. One package can have multiple advisories with different reachability only if Yarn reports them separately; ID is what ignore/suggestion paths already key on.

### 3. When to run the production audit

Compute the production ID set once per final report/`check` emission path that needs annotations (same place unresolved + suggested ignores are formatted), not inside the auto fix loop. Cache on the `Gnarl` run if both unfiltered and filtered paths would otherwise re-hit yarn.

**Why:** Annotation is report-only; the within-range reset loop must not pay an extra audit per iteration.

### 4. Line format

Append a single trailing token after existing fields, space-separated:

- unresolved: `"\"pkg@range\"  prod"` / `"\"pkg@range\"  dev-only"`
- suggested ignores: `"id  severity  pkg@range  prod"` (same token)

**Why:** Stable parseable suffix; keeps package/range quoting intact; clipboard copy of ignore YAML stays separate and unlabeled.

### 5. Failures of the production audit

If the production audit fails, fail the tool run (exit 1) rather than silently omitting labels. Annotation is part of the report contract once shipped; partial unlabeled output would be ambiguous.

## Risks / Trade-offs

- **[Risk] Extra audit latency** → Mitigation: one production audit per report, not per auto iteration; accept cost for annotation-only value.
- **[Risk] Yarn nested-workspace production leaks (berry#6222 / #7255)** → Mitigation: document that labels follow Yarn; do not special-case in gnarl until Yarn’s fix is the baseline users run.
- **[Risk] Advisory in production audit but filtered by severity locally** → Mitigation: classify only advisories already in the severity-thresholded set used for the report; production membership is an ID-set probe, not a second severity pass.

## Migration Plan

No data migration. Operators see new trailing tokens on two section types; bots that scrape those lines may need to tolerate an optional last field. Paste YAML and exit codes unchanged.
