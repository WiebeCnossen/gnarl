## Why

After the auto loop finishes, gnarl may drop unused entries from `package.json` resolutions. Today that rewrite is not followed by `yarn install` / `yarn dedupe`, so `yarn.lock` can stay out of sync with the cleaned resolutions.

## What Changes

- When `reset_resolutions` removes one or more resolutions, run one additional `install` + `dedupe` before the final `check`
- Skip that follow-up when `--no-install` is set (same as the main auto loop)
- Document the extra step in the Auto flow description

## Capabilities

### New Capabilities

- `auto-resolutions-cleanup`: After unused resolutions are dropped from `package.json`, refresh the lockfile with install + dedupe when installs are enabled

### Modified Capabilities

<!-- none - no existing main specs -->

## Impact

- `src/gnarl.rs` (`auto`): honor the dirty flag from `reset_resolutions` and optionally run install/dedupe
- `README.md`: Auto flow steps
- Behavior only; CLI surface unchanged
