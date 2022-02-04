# About

**gnarl** - the yarn v2/v3 companion tool.

# Usage

```
gnarl [<auto | audit | fix | help | reset | shrink> <args>]
```

## Auto

This is the default operation. It will do

1. `yarn install`
2. `yarn dedupe`
3. `gnarl audit`
4. restart from 1 if `yarn.lock` was modified in this iteration

```
gnarl [auto]
```

## Audit

Runs an npm audit,
does `gnarl reset` for issues with a safe fix,
reports remaining issues with suggested resolutions and
checks whether all current resolution are still in use.

```
gnarl audit
```

## Fix

Fixes the resolutions for a package according to the given safe versions.

```
gnarl fix package-name safe-version-request
```

## Help

Prints version and help.

```
gnarl help
```

## Reset

Removes the resolutions for a package, so that a subsequent `yarn install` will update the package.

```
gnarl reset package-names...
```

## Shrink

**DEPRECATED**

Joins package version resolutions, removing old versions where possible.
More aggressive and less reliable than `yarn dedupe`.

```
gnarl shrink
```

# Compilation

```
go build
```
