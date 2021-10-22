# About

**gnarl** - the yarn v2/v3 companion tool.

# Usage

```
gnarl [<auto | fix | help | reset | shrink> <args>]
```

## Auto

This is the default operation. It will do

1. `yarn install`
2. `yarn npm audit --recursive`
3. `gnarl fix` for each of the results
4. `gnarl shrink`
5. restart from 1 if `yarn.lock` was modified in this iteration

```
gnarl [auto]
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

Joins package version resolutions, removing old versions where possible.

```
gnarl shrink
```

# Compilation

```
go build
```
