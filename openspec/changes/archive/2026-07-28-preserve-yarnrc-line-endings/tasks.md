## 1. Line-ending helpers

- [x] 1.1 Add a helper to detect CRLF vs LF from existing `.yarnrc.yml` text (`\r\n` present ⇒ CRLF)
- [x] 1.2 Thread the detected newline through ignore-block generation and splice/join so gnarl-inserted newlines match the file

## 2. Save path

- [x] 2.1 Update `YarnRc::save` (existing-file path) to detect endings from the original text and apply them when rewriting `npmAuditIgnoreAdvisories`
- [x] 2.2 Leave new-file creation on LF; keep stdout `pretty_ignore_block` LF-only unless shared helpers need a newline parameter with LF default

## 3. Tests and lint

- [x] 3.1 Add a regression test: CRLF yarnrc round-trip save keeps CRLF in the rewritten ignore block and does not inject LF-only newlines there
- [x] 3.2 Confirm existing LF yarnrc save tests still pass
- [x] 3.3 Run `cargo clippy --all-targets` and clear any warnings
