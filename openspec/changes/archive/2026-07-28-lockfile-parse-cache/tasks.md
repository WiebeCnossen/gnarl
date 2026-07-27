## 1. Index Locks on load

- [x] 1.1 On `Locks::read`, materialize structured `Lock` entries once into an owned store (keep YAML root for mutation/save)
- [x] 1.2 Build `by_name` and `dependents_of` indexes from those entries
- [x] 1.3 Change `all`, `for_package`, and `dependents` to serve from the store/indexes (no per-call full YAML rematerialize)
- [x] 1.4 After `reset`/`save`, rebuild indexes in place from the updated YAML root so queries stay correct

## 2. Cache Locks on Yarn

- [x] 2.1 Add a `Locks` cache field on `Yarn` and load on first `locks()` call
- [x] 2.2 Adjust `locks()` signature/`&mut self` (or equivalent) so callers reuse the cached instance
- [x] 2.3 Update `gnarl.rs` / `yarn.rs` call sites for the new `locks()` API; ensure post-`reset` queries see updated indexes (in-place rebuild from 1.4 or invalidate+reload)

## 3. Verify

- [x] 3.1 Add or extend tests covering indexed `for_package` / `dependents` and post-`reset` consistency
- [x] 3.2 Confirm `check` / `auto` advisory outcomes are unchanged for a sample project
- [x] 3.3 Run `cargo clippy --all-targets` and clear any warnings
