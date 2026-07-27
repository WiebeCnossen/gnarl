## 1. Auto partial reset

- [x] 1.1 In `Gnarl::fix`, reset when any dependent range is within-range fixable (`fixable`), even if other ranges set `blocked`
- [x] 1.2 Keep parent-advisory escalation for blocked npm dependents in the same pass as a partial reset
- [x] 1.3 Leave `within_range_resettable` on the strict predicate (`fixable && !blocked`) so ignore auto-drop does not fire on partial wins

## 2. Verification

- [x] 2.1 Confirm a mixed fixable/blocked package is reset by `auto` and the fixable range no longer appears under check “fixes” after install
- [x] 2.2 Confirm an ignored advisory with only a partial within-range win keeps its ID in `npmAuditIgnoreAdvisories`
- [x] 2.3 Run `cargo clippy --all-targets` and clear any warnings
