## Context

`npm-audit-ignore-advisories` already prints an enriched overview of current yarnrc ignores and appends `# ignore: <id>` to resolution / unresolved / auto no-fix lines. That inline annotation pollutes copy-paste of those blocks. Users want candidates as a separate section plus paste-ready YAML (new IDs only).

## Goals / Non-Goals

**Goals:**

- Drop every inline `# ignore:` from user-facing output
- Print `suggested ignores` with overview-style enrichment plus a `npmAuditIgnoreAdvisories:` YAML fragment
- Include unresolved issues and outside-range resolution candidates; exclude already-ignored IDs, deprecations, and within-range fixes
- Rename the resolutions section to `suggested resolutions`
- Omit `suggested ignores` when the candidate set is empty

**Non-Goals:**

- Auto-writing suggested IDs into `.yarnrc.yml`
- Merging suggestions with existing ignores in the YAML output (user merges manually)
- Changing ignore overview, auto-drop hygiene, or unfiltered-audit behavior
- Changing how deprecations / fixes sections are formatted (beyond removing any ignore annotations if present)

## Decisions

1. **Collect candidates during `check` classification, print once at the end**  
   Prefer: while building resolutions and unresolved issues, record unique advisory IDs (and enough advisory fields for enrichment). After existing sections, if the filtered set (minus yarnrc ignores) is non-empty, print `suggested ignores` then the YAML block.  
   Alternative: second pass over advisories — unnecessary; classification already knows the buckets.

2. **Enrichment source**  
   Prefer: use the same filtered audit advisories already in hand for `check` (ID, severity, package, vulnerable range). No extra unfiltered audit solely for suggestions.  
   Rationale: suggestions are only for issues still visible under `-s`; overview already uses unfiltered for current yarnrc entries.

3. **YAML formatting**  
   Prefer: reuse `pretty_ignore_block` from `yarnrc.rs` (expose as `pub(crate)` or a small shared helper) so suggestion YAML matches what gnarl writes on save.  
   Alternative: hand-format in `gnarl.rs` — risks drift.

4. **Section title rename**  
   Prefer: change `print_section("resolutions", …)` to `print_section("suggested resolutions", …)` only; line content stays `"pkg@req": "^ver",` without trailing comments.

5. **`auto` no-fix messages**  
   Prefer: print `{}@{} has no fix` without an ID. Final `check()` after auto still emits `suggested ignores` for remaining unresolved / resolution cases.

6. **Dedup**  
   Prefer: one entry per advisory ID in both enriched lines and YAML; skip IDs present in `npmAuditIgnoreAdvisories`; stable sort (e.g. lexicographic ID) for predictable output.

## Risks / Trade-offs

- [User forgets to merge YAML with existing ignores] → Document clearly in README that the block is additions-only
- [Same package yields multiple advisories] → One YAML entry per ID is correct; enrichment lines stay per-ID
- [Severity/package unknown] → Should not happen for classification-sourced IDs; if it did, still list the ID in YAML and mark enrichment unknown (same spirit as overview orphans)

## Migration Plan

- Behavior-only change for `check` / end-of-`auto` stdout; no yarnrc migration
- Update README Check section to describe `suggested ignores` and the section rename

## Open Questions

- None — decisions locked in explore
