## 1. Yarnrc and unfiltered audit

- [x] 1.1 Add a `YarnRc` (or equivalent) reader/writer for `.yarnrc.yml` that lists and removes `npmAuditIgnoreAdvisories` entries and saves the file
- [x] 1.2 Normalize advisory IDs from audit JSON to clean strings (no extra JSON quotes) for compare/display
- [x] 1.3 Spike and implement an unfiltered `yarn npm audit` path that bypasses configured `npmAuditIgnoreAdvisories` without permanently rewriting the project yarnrc
- [x] 1.4 Wire Yarn helpers so callers can obtain ignore IDs from yarnrc and run filtered vs unfiltered audits as needed

## 2. Check output: IDs and overview

- [x] 2.1 Include the audit ID when printing resolution suggestions in `check`
- [x] 2.2 Include the audit ID when printing unresolved / no-fix issues in `check` (and the auto "has no fix" message)
- [x] 2.3 Print an `npmAuditIgnoreAdvisories` overview (ID, package, severity when known) during `check`; handle missing/empty list without failing

## 3. Auto ignore hygiene

- [x] 3.1 After the main auto loop, load yarnrc ignores and unfiltered audit; drop orphan IDs (absent from audit) and save yarnrc
- [x] 3.2 For ignored advisories with a within-range fix, drop the ID, reset the package, and mark the tree dirty for install
- [x] 3.3 When within-range ignore drops caused package resets and installs are enabled, run `install` + `dedupe` (coordinate with existing resolution-cleanup follow-up so work is not duplicated needlessly)
- [x] 3.4 Ensure standalone `check` does not mutate yarnrc; overview may still show orphans as unknown

## 4. Docs

- [x] 4.1 Update README Auto/Check behavior to mention ignore overview, ID hints, and auto-drop of orphan / within-range-fixable ignores
