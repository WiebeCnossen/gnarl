# Spec Delta

## Purpose

Lets `auto` optionally persist suggested audit ignores and report, via process exit status, the highest severity among those newly written IDs so automation can choose push versus review.

## ADDED Requirements

### Requirement: Auto-ignore flag is opt-in on auto only

gnarl MUST accept `--auto-ignore` as a CLI option. The flag MUST apply only to `auto` (including default `auto` when no verb is given). On any other verb, gnarl MUST reject the invocation with a tool-error exit (not a policy exit). Without the flag, `auto` MUST NOT write suggested-ignore candidates.

#### Scenario: Flag on auto is accepted

- **WHEN** the user runs `auto` (or default `auto`) with `--auto-ignore`
- **THEN** gnarl MUST treat the run as an auto-ignore run and MUST NOT fail solely because the flag is present

#### Scenario: Flag on check is rejected

- **WHEN** the user runs `check` with `--auto-ignore`
- **THEN** gnarl MUST fail with a tool-error exit and MUST NOT write `npmAuditIgnoreAdvisories`

#### Scenario: Without the flag auto stays suggestion-only

- **WHEN** `auto` runs without `--auto-ignore`
- **THEN** gnarl MUST NOT add suggested-ignore candidate IDs to `.yarnrc.yml` and a successful run MUST exit 0

### Requirement: Persist suggested ignores after hygiene

On an auto-ignore run, after the existing within-range reset loop and ignore hygiene, and before the final `check`, gnarl MUST merge every suggested-ignore candidate ID into `.yarnrc.yml` `npmAuditIgnoreAdvisories`. Candidates MUST be the same set the final `check` would have suggested without that write. Existing yarnrc entries MUST be kept. Adding ignores MUST NOT by itself trigger install or dedupe.

#### Scenario: New IDs are merged before final check

- **WHEN** an auto-ignore run has at least one suggested-ignore candidate after hygiene
- **THEN** gnarl MUST save those IDs into `npmAuditIgnoreAdvisories` alongside any remaining existing entries before the final `check`

#### Scenario: Final check treats them as current ignores

- **WHEN** those IDs were written this run
- **THEN** the final `check` MUST list them in the current-ignore overview and MUST NOT list them under `suggested ignores`

#### Scenario: No candidates means no ignore write

- **WHEN** an auto-ignore run has no suggested-ignore candidates after hygiene
- **THEN** gnarl MUST NOT add ignore IDs and MUST NOT fail solely because the list is empty

### Requirement: Policy exit encodes max new-ignore severity

A successful auto-ignore run MUST exit 0 when no new ignore IDs were written. When at least one new ID was written, the process MUST exit 10 for info, 11 for low, 12 for moderate, 13 for high, or 14 for critical, using the maximum severity among those newly written IDs. This mapping MUST apply in tagged-stdout mode and after a successful interactive Done quit. Within-range resets and deprecations MUST NOT change the code. A tool or crash failure MUST exit 1 and MUST override 0 and 10–14.

#### Scenario: No new ignores exits zero

- **WHEN** an auto-ignore run completes successfully and writes no new ignore IDs
- **THEN** the process MUST exit 0 even if within-range resets or deprecations occurred

#### Scenario: Highest new ignore is high

- **WHEN** an auto-ignore run newly writes ignore IDs whose severities include high and low
- **THEN** the process MUST exit 13 after a successful tagged-stdout finish or after the user quits a successful interactive Done screen

#### Scenario: Tool failure overrides policy

- **WHEN** an auto-ignore run fails with a tool error after or before writing ignores
- **THEN** the process MUST exit 1 rather than 10–14

### Requirement: Applied-ignore messages include severity

When auto-ignore writes a new ignore ID, gnarl MUST emit a fix/applied message that includes that advisory’s severity.

#### Scenario: Written ignore shows severity

- **WHEN** auto-ignore persists advisory ID `1111111` at severity high
- **THEN** the tagged-stdout and interactive fixes presentation MUST include both the ID and `high`

### Requirement: Advisory-driven reset messages include severity

When `auto` resets a package because of a within-range advisory (including auto-ignore runs), the reset fix message MUST include the severity of that advisory. If one reset is motivated by more than one advisory, the message MUST use the maximum of those severities. A standalone `reset` verb with no advisory context is not required to include a severity.

#### Scenario: Reset for a high advisory

- **WHEN** `auto` resets package `lodash` due to a high within-range advisory
- **THEN** the reset fix message MUST include `high`

#### Scenario: Mixed severities on one package

- **WHEN** `auto` resets one package due to both a moderate and a critical within-range advisory
- **THEN** the reset fix message MUST include `critical`
