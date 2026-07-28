## Context

`YarnRc::save` in `src/yarnrc.rs` splices a rewritten `npmAuditIgnoreAdvisories` block into the existing `.yarnrc.yml` text (preserving comments and unrelated keys). The generated block always uses `\n` via `pretty_ignore_block` and `join_preserving_newlines`, so CRLF files become mixed or partially LF after any save (ignore drop, clear/restore for unfiltered audit, etc.).

Parsing already tolerates CRLF (`trim_yaml_line` strips `\r`), so this is a write-path-only fix.

## Goals / Non-Goals

**Goals:**

- When saving an existing `.yarnrc.yml`, use the same line endings as the rest of that file for the rewritten ignore block and any join newlines gnarl inserts
- Keep LF-only files unchanged in style
- Cover CRLF and LF with unit tests on the save path

**Non-Goals:**

- Changing indentation, quoting, or key order beyond today's splice behavior
- Normalizing mixed line endings across the whole file
- Changing stdout `suggested ignores` YAML (print-only; LF is fine)
- Platform-default endings for brand-new files beyond a simple LF default

## Decisions

1. **Detect style from file contents, not OS**
   Prefer: if the original text contains `\r\n`, treat the file as CRLF; otherwise LF.
   Alternative: always use `std::env::consts` / OS default — rejected; would rewrite LF yarnrcs on Windows.
   Alternative: majority vote per line — unnecessary for typical yarnrc files.

2. **Apply endings only to gnarl-generated segments**
   Prefer: build the ignore block and join separators with the detected newline; leave `before`/`after` substrings untouched (they already carry original endings).
   Implementation sketch: `fn detect_newline(text: &str) -> &'static str`, pass into `pretty_ignore_block` / `join_preserving_newlines` (or convert the LF block with a small replace once before splice).
   Keep the public/stdout `pretty_ignore_block` LF-only unless callers need otherwise.

3. **New file creation stays LF**
   Prefer: when `.yarnrc.yml` does not exist and gnarl creates it with ignores, write LF (matches current behavior and Yarn-ish cross-platform defaults).
   No existing file style to preserve.

4. **Mixed endings**
   Prefer: any `\r\n` ⇒ CRLF for generated text. Do not rewrite untouched regions to “fix” mixed files.

## Risks / Trade-offs

- [File with mixed endings] → Generated block follows CRLF if any CRLF present; rest stays as-is (acceptable; not a normalizer)
- [Tests that assert exact `"\n"` substrings] → Add CRLF-specific tests; existing LF tests should keep passing
- [Accidental `\r\n` → `\r\r\n` double conversion] → Generate with explicit newline string once; do not run a blind global replace on the whole file

## Migration Plan

- No migration; behavior fix on next release
- Rollback: revert the yarnrc write helpers

## Open Questions

<!-- none -->
