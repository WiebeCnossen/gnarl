cd $PSScriptRoot

$cargo = Get-Content -Raw -Path (Join-Path $PSScriptRoot "Cargo.toml")
if ($cargo -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
    throw "Could not read package version from Cargo.toml"
}

$tag = "v$($Matches[1])"
git tag $tag
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
git push --tags
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
