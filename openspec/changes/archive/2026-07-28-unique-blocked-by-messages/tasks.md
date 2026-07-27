## 1. Track printed blocked-by messages

- [x] 1.1 Add a process-lifetime `HashSet` on `Gnarl` for previously printed blocked-by keys (package / other-package / version)
- [x] 1.2 In `fix()`, before printing `{package} blocked by {other-package}@{version}`, insert into the set and skip `out_info!` when the key was already present

## 2. Verify

- [x] 2.1 Confirm fix behavior is unchanged (escalation and resets still run when a message is suppressed)
- [x] 2.2 Run `cargo clippy --all-targets` and clear any warnings
