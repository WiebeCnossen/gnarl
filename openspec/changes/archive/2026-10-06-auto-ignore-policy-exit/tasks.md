# Tasks

## 1. CLI flag

- [x] 1.1 Parse `--auto-ignore` on `Options` and reject it unless the verb is `auto` (including default auto); verify with unit tests that `gnarl --auto-ignore` and `auto --auto-ignore` parse, `check --auto-ignore` / `reset --auto-ignore` error, and `--raw` remains independent
- [x] 1.2 Document `--auto-ignore` and the 0 / 10–14 / 1 exit table in `gnarl help` and README (bot example `gnarl auto --raw --auto-ignore`); verify help text mentions the flag and policy codes

## 2. Process outcome

- [x] 2.1 Add `RunStatus` / policy mapping (`None`→0, info 10 … critical 14) and map `main` success to that code while `Error` stays exit 1; verify with unit tests of the mapping and that encoding 10–14 does not go through `Error`
- [x] 2.2 Plumb `RunStatus` through tagged-stdout `auto`/`check`/`reset` so check and reset-without-auto stay 0 on success; verify existing success paths still compile and tests expect 0 without `--auto-ignore`

## 3. Yarnrc merge

- [x] 3.1 Add merge-new-ignore-IDs on `YarnRc` (keep existing, skip normalized duplicates, reuse `save`); verify unit tests for merge, no duplicate, and CRLF preservation on save

## 4. Apply suggested ignores

- [x] 4.1 Extract shared advisory classification (fixes / resolutions / unresolved / ignore suggestions) used by `check`; verify `check` output sections stay the same in existing formatter/unit tests
- [x] 4.2 After hygiene on `--auto-ignore`, classify, merge new suggestion IDs, emit `Fix` lines that include ID and severity, set policy max from newly written IDs only, then `check`; verify unit tests that no-candidate → exit 0 and no yarnrc add, high+low new IDs → 13, deprecations/resets do not raise the code, and written IDs are omitted from suggested ignores in the following report
- [x] 4.3 Leave `auto` without `--auto-ignore` suggestion-only; verify a test or assertion that yarnrc is unchanged and policy exit is 0

## 5. Reset severity in fix messages

- [x] 5.1 Pass per-package max advisory severity into lockfile reset so auto reset lines include the label (`high`, `critical`, …) while standalone `reset` stays `reset {package}`; verify lock/gnarl tests for single severity, max of mixed severities, and no-severity standalone reset
- [x] 5.2 Apply the same severity-bearing reset messages for hygiene within-range ignore-drop resets without folding them into policy exit; verify the drop/reset message includes severity and policy still ignores it

## 6. Interactive UI

- [x] 6.1 Return `RunStatus` from `run_interactive` after Done + `q` (errors still `Err`); verify UI mode tests still treat `--auto-ignore` as not selecting tagged stdout, and a unit/harness check that a successful worker status 14 is returned after quit rather than forced to 0
- [x] 6.2 Confirm live `fixes` pane shows auto-ignore and reset messages with severity (existing `UiEvent::Fix` text); verify a state/apply test that a fix message containing `high` is recorded in `fixes`

## 7. Clippy

- [x] 7.1 Run `cargo clippy --all-targets` and clear every warning
