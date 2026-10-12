# Tasks

## 1. Classification fixes

- [x] 1.1 In `classify`, pass the lock entry’s tree version into `get_fix` (not `tree_versions().last()`) and verify with a unit test that a lower-range request still classifies as a within-range fix when a higher sibling tree version exists
- [x] 1.2 Prefer `fixes` over `resolutions` for the same package key (invert or replace `fixes.retain`) and verify a unit test keeps the key under `fixes` when both candidates exist

## 2. Resolution-aware within-range apply

- [x] 2.1 When applying a within-range fix, if a matching `package.json` resolution is an exact/blocking pin below the safe version while the dependency request still admits that version, widen the resolution to `^{safe}` (same form as suggested-resolution apply), save, and verify a unit test updates the resolution value
- [x] 2.2 Ensure lockfile reset still runs for caret resolutions that already admit the safe version, and verify `cargo test` covers reset scheduling for that case (or an existing helper assertion)

## 3. Auto loop after hygiene

- [x] 3.1 Restructure `auto` so a hygiene refresh (dropped resolutions and/or ignore-hygiene package resets) re-enters the within-range fix loop before the final `check`, and verify a unit/integration test (or sequenced helper test) that hygiene-dirty implies another fix pass
- [x] 3.2 Confirm the final classification after `auto` has an empty `fixes` map when only outside-range / unresolved work remains; add an assertion or fixture-backed test that documents this invariant

## 4. Quality gate

- [x] 4.1 Run `cargo clippy --all-targets` and clear every warning; run `cargo test` and ensure all tests pass
