## 1. Reproduce and harden yarnrc parsing

- [x] 1.1 Add a unit test that reads `.yarnrc.yml` with unquoted integer `npmAuditIgnoreAdvisories` entries (and a mixed string/integer list) and asserts normalized string IDs
- [x] 1.2 Reproduce any remaining crash/error path beyond `value_to_id` (call sites, save/restore, unfiltered audit) and fix so integer entries never fail solely due to scalar type
- [x] 1.3 Confirm remove + save on an integer-form list rewrites safely (quoted strings OK) and preserves non-ignore yarnrc content

## 2. Hygiene behavior parity

- [x] 2.1 Verify overview / suggested-ignores filtering / auto-drop treat an integer-stored ID the same as the equivalent string ID (extend unit tests or a focused check where practical)
- [x] 2.2 Confirm audit JSON int-or-string ID normalization still matches yarnrc-normalized ignore IDs

## 3. Verification

- [x] 3.1 Run `cargo test` for yarnrc (and related) coverage added above
- [x] 3.2 Run `cargo clippy --all-targets` and clear any warnings
