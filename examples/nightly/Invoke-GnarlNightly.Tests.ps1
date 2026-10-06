#Requires -Version 5.1
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Invoke-GnarlNightly.ps1"

function Assert-Equal {
    param($Actual, $Expected, [string] $Message)
    if ($Actual -ne $Expected) {
        throw "$Message : expected '$Expected', got '$Actual'"
    }
}

function Assert-False {
    param($Actual, [string] $Message)
    if ($Actual) {
        throw "$Message : expected false, got '$Actual'"
    }
}

$thresholdCases = @(
    @{ PrFrom = 'none'; Exit = 0; OpenPr = $false }
    @{ PrFrom = 'none'; Exit = 10; OpenPr = $false }
    @{ PrFrom = 'none'; Exit = 14; OpenPr = $false }
    @{ PrFrom = 'info'; Exit = 0; OpenPr = $false }
    @{ PrFrom = 'info'; Exit = 10; OpenPr = $true }
    @{ PrFrom = 'low'; Exit = 10; OpenPr = $false }
    @{ PrFrom = 'low'; Exit = 11; OpenPr = $true }
    @{ PrFrom = 'moderate'; Exit = 11; OpenPr = $false }
    @{ PrFrom = 'moderate'; Exit = 12; OpenPr = $true }
    @{ PrFrom = 'high'; Exit = 0; OpenPr = $false }
    @{ PrFrom = 'high'; Exit = 12; OpenPr = $false }
    @{ PrFrom = 'high'; Exit = 13; OpenPr = $true }
    @{ PrFrom = 'high'; Exit = 14; OpenPr = $true }
    @{ PrFrom = 'critical'; Exit = 13; OpenPr = $false }
    @{ PrFrom = 'critical'; Exit = 14; OpenPr = $true }
)

foreach ($c in $thresholdCases) {
    $got = Test-GnarlShouldOpenPullRequest -PrFrom $c.PrFrom -ExitCode $c.Exit
    Assert-Equal $got $c.OpenPr "prFrom=$($c.PrFrom) exit=$($c.Exit)"
}

$failed = $false
try {
    Test-GnarlShouldOpenPullRequest -PrFrom 'high' -ExitCode 1 | Out-Null
}
catch {
    $failed = $true
}
if (-not $failed) {
    throw 'exit 1 should throw'
}

$url = Get-GnarlWindowsZipUrl -Version '3.1.0'
Assert-Equal $url 'https://github.com/WiebeCnossen/gnarl/releases/download/v3.1.0/gnarl-x86_64-pc-windows-msvc.zip' 'zip url'
$urlV = Get-GnarlWindowsZipUrl -Version 'v3.1.0'
Assert-Equal $urlV $url 'zip url strips v'

$mrUri = Get-GnarlGitLabMergeRequestsUri -ApiV4 'https://gitlab.example/api/v4/' -ProjectId '42'
Assert-Equal $mrUri 'https://gitlab.example/api/v4/projects/42/merge_requests' 'gitlab mr uri'

$cache = Join-Path ([System.IO.Path]::GetTempPath()) ("gnarl-cache-test-" + [guid]::NewGuid().ToString('n'))
$junkZip = Join-Path ([System.IO.Path]::GetTempPath()) ("not-a-zip-" + [guid]::NewGuid().ToString('n') + '.zip')
Set-Content -LiteralPath $junkZip -Value 'this is not a zip'
New-Item -ItemType Directory -Path $cache | Out-Null
$extractFailed = $false
try {
    Install-GnarlWindowsRelease -Version '3.1.0' -CacheDirectory $cache -ZipPath $junkZip -DryRun
}
catch {
    $extractFailed = $true
}
Assert-False (Test-Path -LiteralPath (Join-Path $cache 'gnarl.exe')) 'DryRun failed extract must not write gnarl.exe'
if (-not $extractFailed) {
    throw 'DryRun with junk zip should fail extract'
}

Remove-Item -LiteralPath $junkZip -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $cache -Recurse -Force -ErrorAction SilentlyContinue

Write-Host 'Invoke-GnarlNightly.Tests.ps1 passed'
