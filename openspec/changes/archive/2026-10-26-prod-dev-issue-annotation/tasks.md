# Tasks

## 1. Production audit + ID set

- [x] 1.1 Add a Yarn path that runs `yarn npm audit --json --recursive --environment production` (reuse existing parse), and verify a unit/integration-style test or CLI dry fixture can distinguish it from the default recursive audit args
- [x] 1.2 Expose a helper that returns the set of advisory IDs from that production audit (normalized string IDs), cache it once per report/`check` emission on the run, and verify a second call does not re-invoke yarn when the cache is warm
- [x] 1.3 Add a pure classifier `prod` vs `dev-only` given an advisory ID and the production ID set, with unit tests for membership and absence

## 2. Annotate report lines

- [x] 2.1 When building `unresolved issues` body lines, carry enough advisory identity to look up the environment label and append `prod` or `dev-only`; verify unit tests cover both labels and that the package@request prefix is unchanged
- [x] 2.2 Append the same trailing label to `suggested ignores` enrichment lines; verify unit tests show the label on enrichment and that `format_ignore_yaml` / paste YAML remains ID-only
- [x] 2.3 Confirm the current `npmAuditIgnoreAdvisories` overview, fixes, suggested resolutions, and hit activity lines stay unlabeled; verify with existing or focused tests that those formats are unchanged
- [x] 2.4 Wire production-audit failure to abort the report/`check` as a tool error; verify a failing production audit surfaces as an error rather than unlabeled lines

## 3. Clippy / regression

- [x] 3.1 Run `cargo clippy --all-targets` and clear every warning introduced by this change
- [x] 3.2 Run the relevant `cargo test` targets for audit/report formatting and confirm they pass
