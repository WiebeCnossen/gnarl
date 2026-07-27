## Context

Today every `Yarn::locks()` call re-reads `yarn.lock` from disk and YAML-parses it into `BTreeMap<String, Value>`. Callers then invoke `Locks::all()`, which re-materializes every entry into `Lock` (parse package names, versions, ranges, dependencies). `for_package` and `dependents` both go through `all()`.

In `check`, that means one full disk+YAML+materialize pass per advisory (plus another for KPI `len()`). In `auto`/`fix`, each advisory does the same for `dependents`, and `create_advisory` often triggers yet another `locks()?.for_package()`. NPM packuments are already cached in-process; the lockfile path is not, so local analysis stays expensive on large lockfiles.

## Goals / Non-Goals

**Goals:**

- Materialize structured lock entries once per load and index them for package lookup and reverse dependents
- Cache the loaded `Locks` on `Yarn` so repeated `locks()` calls in one check / auto iteration reuse the same in-memory view
- Invalidate that cache when gnarl writes the lockfile through `Locks` (e.g. `reset`), so the next `locks()` reloads from disk
- Keep `for_package`, `dependents`, `all`, `len`, and `reset` semantically equivalent to today

**Non-Goals:**

- Using yarn audit `Dependents` instead of the lockfile reverse scan (explore option C — separate decision)
- Changing advisory / fix / resolution algorithms (`get_fix`, `get_resolution`, packument fetch)
- Caching across process runs or watching the lockfile for external edits mid-command
- Micro-optimizations like string-vs-`Version` compare in `check` (nice follow-up, not this change)

## Decisions

### A — Eager materialize + indexes inside `Locks` on read

On `Locks::read`, after YAML deserialize, build once:

- `entries: Vec<Lock>` (or equivalent owned store)
- `by_name: HashMap<String, Vec<usize>>` (or `Vec<&Lock>` / owned slices keyed by package name)
- `dependents_of: HashMap<String, Vec<Dependency>>` reverse index: package → dependents that request it

`all()` returns from `entries`. `for_package(name)` uses `by_name`. `dependents(name)` uses `dependents_of`. Keep the raw YAML root only as needed for `reset`/`save` (mutate map, write file).

After `reset` mutates the YAML root and saves, rebuild indexes from the updated root (or clear and rebuild) so subsequent queries on the same `Locks` instance stay correct without a disk round-trip.

**Alternatives considered:**

- Lazy `OnceCell` on first `all()` — slightly deferred cost, but every advisory path hits `all()` immediately; eager is simpler
- Index only by name, rebuild dependents on each call — still O(L) per advisory for the hot `fix` path
- Replace YAML root with structured-only representation — larger rewrite; keep YAML for save fidelity

### B — Cache `Locks` on `Yarn`

`Yarn` holds `Option<Locks>` (or similar). `locks()` returns a cached instance after first load. Provide an explicit invalidate (or have mutating APIs go through Yarn) so after `locks()?.reset(...)` the cache is cleared and the next `locks()` reloads the written file.

Preferred shape: `Yarn::locks(&mut self) -> Result<&mut Locks, Error>` (or interior mutability) so reset can happen on the cached object, then either rebuild indexes in place after save, or invalidate for a fresh read. In-place rebuild after `reset` avoids an immediate re-read of what we just wrote.

**Alternatives considered:**

- Leave caching to call sites (`let locks = yarn.locks()?;` once in `check`) — easy to regress; central cache matches “always cheap”
- Process-global static cache — wrong if cwd/`yarn.lock` changes; Yarn-scoped is enough
- File mtime checks on every `locks()` — overkill for one command run; explicit invalidate on our writes is enough

### Invalidation policy

| Event | Action |
|-------|--------|
| First `locks()` in a `Yarn` lifetime | Load + index |
| Later `locks()` same `Yarn` | Return cache |
| `Locks::reset` / `save` on cached instance | Rebuild indexes in place (preferred) or invalidate Yarn cache |
| New `Yarn::new` (e.g. next auto outer loop after install) | Fresh load — correct because install may rewrite lockfile |

New `Yarn` each outer `auto` iteration already reloads after install; no extra invalidation needed there.

## Risks / Trade-offs

- [Risk] Stale cache if something else writes `yarn.lock` while a `Yarn` is live → Mitigation: gnarl's own writers go through `Locks`; external tools during a single command are out of scope
- [Risk] Index rebuild bugs after `reset` diverge from a full re-read → Mitigation: rebuild from the same materialize path used on read; add a focused unit/integration check that `for_package`/`dependents` after reset match expectations
- [Risk] Higher peak memory (YAML root + indexes) → Accepted; same data was already rebuilt repeatedly; peak is one copy of what we used to allocate N times
- [Trade-off] `locks()` may need `&mut self` → Call sites in `gnarl.rs` / `yarn.rs` adjust; behavior unchanged

## Migration Plan

No user migration. Pure internal performance change; no lockfile or config format changes.

## Open Questions

None — A+B scope confirmed from explore; audit-dependents reuse deferred.
