## Context

`.yarnrc.yml` `npmAuditIgnoreAdvisories` is a YAML sequence. Yarn accepts both quoted strings (`"1090865"`) and unquoted integers (`1090865`). Teams paste either form. gnarl reads yarnrc via `serde_yaml::Value` and already has a `value_to_id` helper that maps `String` and `Number` to `String`, but integer-form lists still fail in practice (user-reported crash), and there is no regression coverage for that input shape. Audit JSON `ID` is already normalized from int-or-string via `normalize_advisory_id`.

## Goals / Non-Goals

**Goals:**

- Read `npmAuditIgnoreAdvisories` entries written as YAML strings or integers without error
- Normalize both to the same decimal string so overview, suggested-ignores filtering, remove, and auto-drop match Yarn/audit IDs
- Preserve existing save format (quoted strings via `pretty_ignore_block`)
- Lock the behavior with unit tests on mixed string/integer yarnrc lists

**Non-Goals:**

- Changing how gnarl writes ignore IDs (keep quoted strings)
- Accepting arbitrary YAML types (maps, nulls, floats used as IDs) beyond string and integer
- Changing Yarn itself or inventing a new ignore config location
- Altering ignore overview / suggest / auto-drop product rules beyond format tolerance

## Decisions

1. **Normalize at the yarnrc read boundary**  
   Prefer: coerce each sequence item to `String` in one place (`value_to_id` or equivalent) so every caller sees `Vec<String>`.  
   Alternative: typed `enum Id { String, Int }` throughout — rejected; adds noise with no product benefit.

2. **Integer → string via decimal `Display` of the YAML number**  
   Prefer: `Number::to_string()` (same idea as audit `normalize_advisory_id`) so `1090865` becomes `"1090865"`.  
   Alternative: always re-quote through serde — unnecessary if we only need canonical string equality with audit IDs.

3. **On save, always emit quoted strings**  
   Prefer: keep `pretty_ignore_block` / splice writer as-is. Reading integers then saving may rewrite those entries as strings; that is Yarn-compatible and clearer than preserving original scalar style.  
   Alternative: round-trip original quoting — rejected; text-splice writer already rewrites the whole ignore block.

4. **Reproduce then harden**  
   Prefer: add a failing regression test with unquoted integer list items (and mixed string/int), fix any remaining panic/error path (e.g. a call site that still assumes `as_str()`-only), then keep the tests green.  
   Do not assume current `value_to_id` is sufficient without proving end-to-end read → remove → save and unfiltered-audit restore.

## Risks / Trade-offs

- [Very large numeric IDs / YAML float parsing] → Stick to integer scalars Yarn emits; if a value is not a clean integer string after coerce, treat as opaque string only when it was already a YAML string (GHSA ids). Do not invent float→int conversion.
- [Silent drop of unsupported scalars] → Prefer clear skip or error only for truly invalid items; string and integer MUST succeed. Document that non-scalar items remain ignored as today.
- [Rewrite integers to strings on save] → Acceptable; matches Yarn docs/examples and gnarl’s existing writer.
