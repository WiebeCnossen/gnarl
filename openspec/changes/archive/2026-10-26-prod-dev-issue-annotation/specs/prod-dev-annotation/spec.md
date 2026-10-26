# Spec Delta

## Purpose

Annotate unresolved audit issues and suggested-ignore enrichment lines with whether each advisory is reachable via production dependencies or only via:devDependencies, without changing remediation, ignore policy, or paste YAML.

## ADDED Requirements

### Requirement: Production vs:dev-only classification

For each advisory that appears in the severity-thresholded recursive audit used for `check` classification, gnarl MUST classify it as `prod` when that advisory ID also appears in a recursive production-environment audit, and as `dev-only` when it appears in the full recursive audit but not in the production-environment audit. Classification MUST use Yarn’s production-environment semantics (equivalent to `yarn npm audit --environment production`).

#### Scenario: Advisory in both audits is prod

- **WHEN** an advisory ID is present in both the full recursive audit and the production-environment recursive audit
- **THEN** gnarl MUST classify that advisory as `prod`

#### Scenario: Advisory only in full audit is:dev-only

- **WHEN** an advisory ID is present in the full recursive audit and absent from the production-environment recursive audit
- **THEN** gnarl MUST classify that advisory as `dev-only`

### Requirement: Unresolved issues lines include environment label

Each body line in the `unresolved issues` section MUST end with a space-separated environment label `prod` or `dev-only` for the advisory that produced that line. The package and request portion of the line MUST otherwise keep the existing form. This applies under tagged-stdout mode and on the interactive Done screen (same body strings).

#### Scenario: Unresolved prod issue annotated

- **WHEN** `check` presents an unresolved issue for an advisory classified as `prod`
- **THEN** that unresolved body line MUST end with `prod`

#### Scenario: Unresolved:dev-only issue annotated

- **WHEN** `check` presents an unresolved issue for an advisory classified as `dev-only`
- **THEN** that unresolved body line MUST end with `dev-only`

### Requirement: Annotation does not change policy or other sections

Environment labels MUST NOT change which advisories are classified as fixes, suggested resolutions, unresolved, or ignore candidates; MUST NOT change exit codes or auto-ignore writes; and MUST NOT appear on paste-ready `npmAuditIgnoreAdvisories` YAML, `suggested resolutions` / `fixes` body lines, activity hit lines, or the current `npmAuditIgnoreAdvisories` overview.

#### Scenario: Paste YAML stays ID-only

- **WHEN** `check` emits suggested-ignore YAML
- **THEN** that YAML MUST list advisory IDs without `prod` or `dev-only` tokens

#### Scenario: Overview stays without environment label

- **WHEN** `check` presents the current `npmAuditIgnoreAdvisories` overview
- **THEN** those enrichment lines MUST NOT include `prod` or `dev-only`
