## Why

When `auto` loops after a reset (including partial within-range resets), the same `{package} blocked by {other-package}@{version}` info line is reprinted on later iterations even though nothing new happened. That noise makes it harder to see what actually changed in the run.

## What Changes

- Deduplicate `{package} blocked by {other-package}@{version}` messages so each distinct message is printed at most once per `gnarl` process run
- Dedup applies across outer `auto` loop iterations; fix behavior (escalation, resets) is unchanged
- Within a single `fix()` pass, identical lines from multiple dependents may still appear (out of scope)

## Capabilities

### New Capabilities

- `unique-blocked-by-messages`: Ensure each `{package} blocked by {other-package}@{version}` info message is shown at most once per program run across `auto` iterations

### Modified Capabilities

- (none)

## Impact

- `src/gnarl.rs`: track previously printed blocked-by messages on `Gnarl` and gate `out_info!` accordingly
- No CLI, lockfile, or yarn behavior changes
