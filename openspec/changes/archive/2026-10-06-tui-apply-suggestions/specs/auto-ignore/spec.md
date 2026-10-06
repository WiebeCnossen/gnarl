# Spec Delta

## ADDED Requirements

### Requirement: Interactive ignore apply is not a policy auto-ignore run

Persisting suggested ignores because the operator pressed `I` on the interactive Done screen MUST NOT by itself select policy exits 10–14. A successful session that wrote ignores only via `I` MUST exit 0 after quit, unless a later `--auto-ignore` `auto` continuation wrote new IDs under the existing auto-ignore policy rules. Adding ignores via `I` MUST NOT by itself trigger install or dedupe.

#### Scenario: I on check exits zero

- **WHEN** interactive `check` applies suggested ignores via `I`, reaches Done, and the user presses `q`
- **THEN** the process MUST exit 0

#### Scenario: I on auto without the flag exits zero

- **WHEN** interactive `auto` without `--auto-ignore` applies suggested ignores via `I`, reaches Done, and the user presses `q`
- **THEN** the process MUST exit 0 even if newly written IDs include high or critical severity

### Requirement: Interactive ignore apply uses the same candidates and merge

When `I` applies ignores, gnarl MUST merge every current suggested-ignore candidate ID into `.yarnrc.yml` `npmAuditIgnoreAdvisories`, keep existing entries, and use the same candidate set the Done screen would have suggested for copy. `I` MUST be accepted on interactive `check` and `auto` Done screens.

#### Scenario: I on check writes yarnrc

- **WHEN** interactive `check` Done has suggested-ignore candidates and the operator presses `I`
- **THEN** gnarl MUST merge those IDs into `npmAuditIgnoreAdvisories` and MUST NOT fail solely because the verb is `check`

## MODIFIED Requirements

### Requirement: Auto-ignore flag is opt-in on auto only

gnarl MUST accept `--auto-ignore` as a CLI option. The flag MUST apply only to `auto` (including default `auto` when no verb is given). On any other verb, gnarl MUST reject the invocation with a tool-error exit (not a policy exit). Without the flag, `auto` MUST NOT write suggested-ignore candidates during the `auto` pipeline; the interactive Done apply-ignores key MAY write those candidates afterward as specified under interactive ignore apply.

#### Scenario: Flag on auto is accepted

- **WHEN** the user runs `auto` (or default `auto`) with `--auto-ignore`
- **THEN** gnarl MUST treat the run as an auto-ignore run and MUST NOT fail solely because the flag is present

#### Scenario: Flag on check is rejected

- **WHEN** the user runs `check` with `--auto-ignore`
- **THEN** gnarl MUST fail with a tool-error exit and MUST NOT write `npmAuditIgnoreAdvisories`

#### Scenario: Without the flag auto stays suggestion-only

- **WHEN** `auto` runs without `--auto-ignore` and the operator does not activate the interactive apply-ignores key
- **THEN** gnarl MUST NOT add suggested-ignore candidate IDs to `.yarnrc.yml` and a successful run MUST exit 0
