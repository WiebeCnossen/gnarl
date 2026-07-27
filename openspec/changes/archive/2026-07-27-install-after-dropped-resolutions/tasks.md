## 1. Auto follow-up after resolution cleanup

- [x] 1.1 In `Gnarl::auto`, capture the `bool` returned by `reset_resolutions`
- [x] 1.2 When that flag is true and `--no-install` is not set, run `Yarn::install` then `Yarn::dedupe` before `check`
- [x] 1.3 When the flag is false, or `--no-install` is set, skip the follow-up install/dedupe

## 2. Docs

- [x] 2.1 Update README Auto flow to mention dropping unused resolutions and the conditional extra install + dedupe
