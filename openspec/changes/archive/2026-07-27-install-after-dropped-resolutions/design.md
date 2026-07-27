## Context

`Gnarl::auto` loops `install` → `dedupe` → audit/fix until the lockfile stops changing, then calls `Yarn::reset_resolutions`. That method returns `true` when it drops unused `package.json` resolutions and saves the file. The return value was ignored, so the lockfile was never refreshed after cleanup.

## Goals / Non-Goals

**Goals:**

- After resolutions are dropped, run one more `install` + `dedupe` so `yarn.lock` matches `package.json`
- Respect `--no-install` the same way the main loop does
- Keep the follow-up outside the audit loop (no extra audit cycle solely because resolutions were dropped)

**Non-Goals:**

- Changing when a resolution is considered unused (`reset_resolutions` heuristics stay as-is)
- Re-entering the full auto loop after dropping resolutions
- Changing `gnarl reset` beyond whatever it inherits via `auto`

## Decisions

1. **Use the existing dirty flag from `reset_resolutions`**  
   Prefer: `let dirty = yarn.reset_resolutions()?; if dirty && !no_install { install; dedupe }`  
   Over: always running a final install/dedupe (wastes work when nothing was dropped)  
   Or: folding cleanup into the loop condition (would re-audit; out of scope)

2. **One-shot follow-up, not another loop iteration**  
   Dropping unused resolutions should only refresh the tree, not restart advisory fixing. If a later need for re-audit appears, that is a separate change.

3. **Reuse `Yarn::install` / `Yarn::dedupe`**  
   Same code path as the loop (including aikido/safe-chain preference on install).

## Risks / Trade-offs

- [Extra yarn cost when many resolutions are dropped] → Acceptable; only runs when dirty  
- [Dropped resolution could surface new advisories that check reports but auto does not re-fix] → Intentional for this change; document as non-goal
