# About

**gnarl** - the yarn v2/v3/v4 companion tool (Rust implementation).

This is a complete and incompatible rewrite of the Go version.

*Note*: it is highly recommended to use `yarn` version `4.15` or later with `npmMinimalAgeGate: 1d` in the `.yarnrc.yml` file in combination with Aikido safe-chain.

# Usage

```
gnarl [check | reset <packages>] [-s <severity>]
```

## Auto

This is the default operation. It will do

1. `install`
2. `dedupe`
3. `audit`
4. `reset` packages that can be fixed within the specified range
5. restart from 1 if `yarn.lock` was modified in this iteration
6. drop unused resolutions from `package.json`
7. drop orphan `npmAuditIgnoreAdvisories` entries and entries superseded by a within-range fix (resetting those packages)
8. if resolutions were removed or ignore hygiene reset packages, run `install` + `dedupe` once more
9. run `check` (including ignore overview and ID hints)

```
gnarl [-s <severity>]
```

## Check

Only runs an audit and checks what issues and fixes are available. It also:

- prints an overview of current `.yarnrc.yml` `npmAuditIgnoreAdvisories` (ID, severity, package when known)
- mentions the Yarn audit ID next to resolution suggestions and unresolved / no-fix issues (candidate for `npmAuditIgnoreAdvisories`)

`check` does not modify `.yarnrc.yml`.

```
gnarl check [-s severity]
```

## Reset

Removes the resolutions for a package, so that a subsequent `yarn install` will update the package.

```
gnarl reset package-names...
```

## Help

Prints version and help.

```
gnarl help
```

# Compilation

```
cargo build --release
```

The binary will be in `target/release/gnarl` (or `target/release/gnarl.exe` on Windows).
