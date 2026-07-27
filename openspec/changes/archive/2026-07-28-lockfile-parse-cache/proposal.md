## Why

Advisory analysis re-reads and re-parses `yarn.lock` for every advisory (`yarn.locks()` → `Locks::all()` via `for_package` / `dependents`). On large lockfiles that cost dominates even when NPM packuments are already cached, so `check` and the local part of `auto` feel slower than they need to be.

## What Changes

- Parse `yarn.lock` into structured `Lock` data once, and keep indexes for package lookup and reverse dependents
- Cache the parsed `Locks` on `Yarn` for the lifetime of a check / auto iteration (invalidate after lockfile mutations that write through `Locks`)
- Leave advisory outcomes, resolutions, and fix/reset behavior unchanged — this is a performance change only

## Capabilities

### New Capabilities

- `lockfile-parse-cache`: Parse and index the lockfile once per load, cache it on `Yarn`, and serve `for_package` / `dependents` / `all` from that cache without repeating disk reads or full re-materialization

### Modified Capabilities

- (none)

## Impact

- `src/locks.rs`: materialize and index locks on read; use indexes in `for_package` / `dependents` / `all`; invalidate or rebuild indexes after `reset`/`save` as needed
- `src/yarn.rs`: cache `Locks` across repeated `locks()` calls until the lockfile must be reloaded
- Call sites in `src/gnarl.rs` stay behaviorally the same; may simplify slightly once caching is reliable
- No CLI, audit, or package.json format changes
