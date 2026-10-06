# Spec Delta

## Purpose

Lets an interactive operator persist the current outside-range suggested resolutions into `package.json` and continue with a full `auto` run so the lockfile and audit match the new pins.

## ADDED Requirements

### Requirement: Apply writes every suggested resolution

When the operator applies suggested resolutions from the interactive Done screen, gnarl MUST write every currently suggested outside-range resolution into `package.json` `resolutions`. Each entry MUST use the same package key and caret version as the corresponding `suggested resolutions` body line. Existing unrelated `resolutions` entries MUST be kept. gnarl MUST NOT parse clipboard or tagged-stdout text to obtain those keys.

#### Scenario: All current suggestions are persisted

- **WHEN** the Done screen has at least one suggested resolution and the operator activates the apply-resolutions key
- **THEN** gnarl MUST add each suggested key/value to `package.json` `resolutions` without removing other resolution entries

#### Scenario: Empty suggestions do not write

- **WHEN** the suggested resolutions section is empty and the operator activates the apply-resolutions key
- **THEN** gnarl MUST NOT modify `package.json`

### Requirement: Apply is followed by a full auto with opening refresh

After a successful suggested-resolution write, gnarl MUST run a full `auto` (within-range reset loop, hygiene, then `check`). That `auto` MUST open with `yarn install` then `yarn dedupe` even when `-x` / `--install-on-change` was set on the original invocation, because this process already mutated `package.json`. `--auto-ignore` on the original `auto` MUST still apply to this continuation; an original `check` MUST NOT gain `--auto-ignore`.

#### Scenario: Continuation refreshes then autos

- **WHEN** suggested resolutions were written from Done
- **THEN** gnarl MUST run install, dedupe, and the rest of `auto` before presenting a new Done screen

#### Scenario: Install-on-change still refreshes after the write

- **WHEN** the original invocation used `-x` or `--install-on-change` and suggested resolutions were written from Done
- **THEN** the continuation `auto` MUST still run install then dedupe before its first audit
