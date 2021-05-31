# About

**gnarl** - the yarn v2 companion tool.

# Usage

```
gnarl <fix | help | reset | shrink> <args>
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