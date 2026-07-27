## Purpose

Parse and index `yarn.lock` once per load, cache it on `Yarn`, and serve package/dependents lookups without repeating disk reads or full rematerialization—without changing advisory analysis outcomes.

## Requirements

### Requirement: Lockfile is materialized and indexed once per load

When gnarl loads `yarn.lock` into `Locks`, it MUST build structured lock entries and lookup indexes once for that load. Subsequent `all`, `for_package`, and `dependents` calls on that instance MUST use the indexed data and MUST NOT re-parse the entire lockfile from YAML values on each call.

#### Scenario: Repeated package lookups on one Locks instance

- **WHEN** `Locks` has been loaded from disk and `for_package` is called for multiple package names
- **THEN** each lookup returns the same packages that a full scan would, without re-materializing every lock entry from the raw YAML map again

#### Scenario: Repeated dependents queries on one Locks instance

- **WHEN** `Locks` has been loaded from disk and `dependents` is called for multiple package names
- **THEN** each query returns the dependents that request that package, using a reverse index (or equivalent) built at load time rather than scanning every package's dependency list from scratch via a full re-materialize

### Requirement: Yarn caches loaded Locks until reload is required

`Yarn` MUST cache the loaded `Locks` so repeated `locks()` calls within the same `Yarn` lifetime reuse one in-memory instance instead of re-reading `yarn.lock` from disk each time.

#### Scenario: Multiple locks() calls during check

- **WHEN** `check` (or equivalent analysis) calls `locks()` more than once on the same `Yarn` without an intervening lockfile write by gnarl
- **THEN** only the first call reads and parses `yarn.lock` from disk; later calls reuse the cached `Locks`

#### Scenario: Fresh Yarn after install reloads lockfile

- **WHEN** a new `Yarn` is constructed after an install that may have rewritten `yarn.lock`
- **THEN** the next `locks()` load reads the current file from disk (no cross-`Yarn` cache)

### Requirement: Indexes stay correct after lockfile reset

After gnarl resets packages through `Locks` and saves `yarn.lock`, subsequent queries on the active locks view MUST reflect the post-reset contents (removed package entries gone from `for_package` / `all` / `dependents`).

#### Scenario: Query after reset on cached locks

- **WHEN** `reset` removes one or more packages from the lockfile and saves
- **THEN** the next `for_package` / `dependents` / `all` results match the updated lockfile (via in-place index rebuild or cache invalidation and reload)

### Requirement: Advisory analysis behavior is unchanged

Caching and indexing MUST NOT change which fixes, resolutions, errors, resets, or blocked-by outcomes advisory analysis produces for a given project state.

#### Scenario: Same analysis outcomes

- **WHEN** `check` or `auto` runs against a project after this change
- **THEN** suggested fixes, resolutions, unresolved issues, and reset decisions match the pre-change behavior for the same inputs (modulo performance and incidental log timing)
