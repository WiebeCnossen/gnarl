#Requires -Version 5.1
<#
.SYNOPSIS
  Nightly wrapper around gnarl auto for Azure DevOps or GitLab (not a gnarl feature).

.NOTES
  Windows zip URL (cargo-dist, tag v{version}):
  https://github.com/WiebeCnossen/gnarl/releases/download/v3.1.0/gnarl-x86_64-pc-windows-msvc.zip
  Asset name is always gnarl-x86_64-pc-windows-msvc.zip. Pin gnarlVersion; do not use latest.
#>
[CmdletBinding()]
param(
    [ValidateSet('EnsureBinary', 'Auto')]
    [string] $Mode = 'Auto',

    [string] $GnarlExe = '',
    [string] $YarnWorkingDirectory = '.',
    [string] $PrFrom = 'high',
    [string] $DefaultBranch = 'main',
    [string] $GnarlVersion = '3.1.0',
    [string] $CacheDirectory = '',
    [string] $RepoRoot = '',
    [ValidateSet('AzureDevOps', 'GitLab')]
    [string] $GitHost = 'AzureDevOps'
)

$script:GnarlFixBranch = 'gnarl/audit-ignores'
$script:GnarlWindowsAsset = 'gnarl-x86_64-pc-windows-msvc.zip'
$script:GnarlReleasesOrgRepo = 'WiebeCnossen/gnarl'

function Get-GnarlNormalizedVersion {
    param([Parameter(Mandatory)][string] $Version)
    return $Version.Trim().TrimStart('v', 'V')
}

function Get-GnarlWindowsZipUrl {
    param([Parameter(Mandatory)][string] $Version)
    $v = Get-GnarlNormalizedVersion -Version $Version
    return "https://github.com/$script:GnarlReleasesOrgRepo/releases/download/v$v/$script:GnarlWindowsAsset"
}

function Get-GnarlPrThresholdCode {
    param([Parameter(Mandatory)][string] $PrFrom)
    switch ($PrFrom.ToLowerInvariant()) {
        'none' { return $null }
        'info' { return 10 }
        'low' { return 11 }
        'moderate' { return 12 }
        'high' { return 13 }
        'critical' { return 14 }
        default { throw "Unknown prFrom '$PrFrom'. Use none, info, low, moderate, high, or critical." }
    }
}

function Test-GnarlShouldOpenPullRequest {
    param(
        [Parameter(Mandatory)][string] $PrFrom,
        [Parameter(Mandatory)][int] $ExitCode
    )
    if ($ExitCode -eq 1) {
        throw 'gnarl exited 1 (tool error); do not push or open a PR.'
    }
    if ($ExitCode -eq 0) {
        return $false
    }
    $threshold = Get-GnarlPrThresholdCode -PrFrom $PrFrom
    if ($null -eq $threshold) {
        return $false
    }
    if ($ExitCode -lt 10 -or $ExitCode -gt 14) {
        throw "Unexpected gnarl exit code $ExitCode."
    }
    return $ExitCode -ge $threshold
}

function Get-GnarlScopedPaths {
    param([Parameter(Mandatory)][string] $YarnWorkingDirectory)
    $names = @('package.json', 'yarn.lock', '.yarnrc.yml')
    return @($names | ForEach-Object { Join-Path $YarnWorkingDirectory $_ })
}

function Install-GnarlWindowsRelease {
    param(
        [Parameter(Mandatory)][string] $Version,
        [Parameter(Mandatory)][string] $CacheDirectory,
        [string] $ZipPath = '',
        [switch] $DryRun
    )
    $exePath = Join-Path $CacheDirectory 'gnarl.exe'
    if ((Test-Path -LiteralPath $exePath) -and -not $DryRun) {
        return $exePath
    }

    $url = Get-GnarlWindowsZipUrl -Version $Version
    $tempRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("gnarl-extract-" + [guid]::NewGuid().ToString('n'))
    New-Item -ItemType Directory -Path $tempRoot | Out-Null
    $ownsZip = $false
    try {
        if (-not $ZipPath) {
            if ($DryRun) {
                Write-Host "DryRun: would download $url then extract to cache only after success."
                return $url
            }
            $ZipPath = Join-Path $tempRoot $script:GnarlWindowsAsset
            Write-Host "Downloading $url"
            Invoke-WebRequest -Uri $url -OutFile $ZipPath -UseBasicParsing
            $ownsZip = $true
        }

        $extractDir = Join-Path $tempRoot 'out'
        New-Item -ItemType Directory -Path $extractDir | Out-Null
        try {
            Expand-Archive -LiteralPath $ZipPath -DestinationPath $extractDir -Force
        }
        catch {
            Write-Warning "Extract failed; cache directory will not be written. $($_.Exception.Message)"
            if (Test-Path -LiteralPath $CacheDirectory) {
                Remove-Item -LiteralPath $exePath -ErrorAction SilentlyContinue
            }
            throw
        }

        $found = Get-ChildItem -LiteralPath $extractDir -Filter 'gnarl.exe' -Recurse -File | Select-Object -First 1
        if (-not $found) {
            throw "Zip did not contain gnarl.exe: $ZipPath"
        }

        if ($DryRun) {
            Write-Host "DryRun: extract succeeded; not writing $exePath"
            return $found.FullName
        }

        New-Item -ItemType Directory -Path $CacheDirectory -Force | Out-Null
        Copy-Item -LiteralPath $found.FullName -Destination $exePath -Force
        return $exePath
    }
    finally {
        if ($ownsZip -or (Test-Path -LiteralPath $tempRoot)) {
            Remove-Item -LiteralPath $tempRoot -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

function Write-GnarlHost {
    param([Parameter(Mandatory)][string] $Message)
    Write-Host $Message
}

function Invoke-GnarlGit {
    param([Parameter(Mandatory)][string[]] $GitArgs)
    Write-GnarlHost ("git " + ($GitArgs -join ' '))
    $out = & git @GitArgs 2>&1
    $code = $LASTEXITCODE
    foreach ($line in @($out)) {
        Write-Host $line
    }
    if ($code -ne 0) {
        throw "git $($GitArgs -join ' ') failed with $code"
    }
}

function Switch-GnarlToDefaultBranch {
    param([Parameter(Mandatory)][string] $DefaultBranch)
    Invoke-GnarlGit -GitArgs @('fetch', 'origin', $DefaultBranch)
    Invoke-GnarlGit -GitArgs @('checkout', '-B', $DefaultBranch, "origin/$DefaultBranch")
}

function Invoke-GnarlLogged {
    param(
        [Parameter(Mandatory)][string] $GnarlExe,
        [Parameter(Mandatory)][string] $WorkingDirectory
    )
    $stdoutPath = Join-Path ([System.IO.Path]::GetTempPath()) ("gnarl-stdout-" + [guid]::NewGuid().ToString('n') + '.txt')
    $stderrPath = Join-Path ([System.IO.Path]::GetTempPath()) ("gnarl-stderr-" + [guid]::NewGuid().ToString('n') + '.txt')
    Write-GnarlHost "----- gnarl auto --raw --auto-ignore --install-on-change -----"
    $proc = Start-Process -FilePath $GnarlExe -ArgumentList @('auto', '--raw', '--auto-ignore', '--install-on-change') -WorkingDirectory $WorkingDirectory -NoNewWindow -Wait -PassThru -RedirectStandardOutput $stdoutPath -RedirectStandardError $stderrPath
    if (Test-Path -LiteralPath $stdoutPath) {
        Get-Content -LiteralPath $stdoutPath | ForEach-Object { Write-Host $_ }
    }
    if ((Test-Path -LiteralPath $stderrPath) -and ((Get-Item -LiteralPath $stderrPath).Length -gt 0)) {
        Write-GnarlHost '----- gnarl stderr -----'
        Get-Content -LiteralPath $stderrPath | ForEach-Object { Write-Host $_ }
    }
    Write-GnarlHost "----- gnarl exit $($proc.ExitCode) -----"
    Remove-Item -LiteralPath $stdoutPath, $stderrPath -ErrorAction SilentlyContinue
    return $proc.ExitCode
}

function Get-GnarlGitLabMergeRequestsUri {
    param(
        [Parameter(Mandatory)][string] $ApiV4,
        [Parameter(Mandatory)][string] $ProjectId
    )
    return "$($ApiV4.TrimEnd('/'))/projects/$ProjectId/merge_requests"
}

function Publish-GnarlAzureDevOps {
    param(
        [Parameter(Mandatory)][bool] $OpenPr,
        [Parameter(Mandatory)][string] $DefaultBranch,
        [Parameter(Mandatory)][int] $ExitCode,
        [Parameter(Mandatory)][string] $PrFrom
    )
    if ($OpenPr) {
        if ($env:SYSTEM_ACCESSTOKEN) {
            $env:AZURE_DEVOPS_EXT_PAT = $env:SYSTEM_ACCESSTOKEN
        }
        Invoke-GnarlGit -GitArgs @('checkout', '-B', $script:GnarlFixBranch)
        Invoke-GnarlGit -GitArgs @('push', '-u', 'origin', $script:GnarlFixBranch, '--force')
        $azArgs = @(
            'repos', 'pr', 'create',
            '--source-branch', $script:GnarlFixBranch,
            '--target-branch', $DefaultBranch,
            '--title', 'gnarl auto (review ignores)',
            '--description', "Nightly gnarl auto; policy exit $ExitCode (prFrom=$PrFrom).",
            '--output', 'json'
        )
        Write-GnarlHost "az $($azArgs -join ' ')"
        $azOut = & az @azArgs 2>&1 | Out-String
        Write-Host $azOut
        if ($LASTEXITCODE -ne 0) {
            Write-GnarlHost "az repos pr create exited $LASTEXITCODE (branch $script:GnarlFixBranch pushed; an open PR targeting $DefaultBranch may already exist)."
            return
        }
        try {
            $pr = $azOut | ConvertFrom-Json
            $web = $null
            if ($env:SYSTEM_COLLECTIONURI -and $env:SYSTEM_TEAMPROJECT -and $env:BUILD_REPOSITORY_NAME -and $pr.pullRequestId) {
                $web = "$($env:SYSTEM_COLLECTIONURI)$($env:SYSTEM_TEAMPROJECT)/_git/$($env:BUILD_REPOSITORY_NAME)/pullrequest/$($pr.pullRequestId)"
            }
            Write-GnarlHost "Pull request $($pr.pullRequestId) : $script:GnarlFixBranch -> $DefaultBranch (policy exit $ExitCode, prFrom=$PrFrom)"
            if ($web) {
                Write-GnarlHost $web
            }
        }
        catch {
            Write-GnarlHost "Branch $script:GnarlFixBranch pushed for review targeting $DefaultBranch (policy exit $ExitCode, prFrom=$PrFrom)."
        }
    }
    else {
        Invoke-GnarlGit -GitArgs @('push', 'origin', "HEAD:$DefaultBranch")
        Write-GnarlHost "Pushed to $DefaultBranch (policy exit $ExitCode, prFrom=$PrFrom; no PR)."
    }
}

function Publish-GnarlGitLab {
    param(
        [Parameter(Mandatory)][bool] $OpenPr,
        [Parameter(Mandatory)][string] $DefaultBranch,
        [Parameter(Mandatory)][int] $ExitCode,
        [Parameter(Mandatory)][string] $PrFrom
    )
    $token = $env:GNARL_GITLAB_TOKEN
    if (-not $token) {
        throw 'Set CI/CD variable GNARL_GITLAB_TOKEN (project access token with api and write_repository).'
    }
    if (-not $env:CI_SERVER_URL -or -not $env:CI_PROJECT_PATH) {
        throw 'GitLab publish requires CI_SERVER_URL and CI_PROJECT_PATH.'
    }
    $origin = "$($env:CI_SERVER_URL.TrimEnd('/'))/$($env:CI_PROJECT_PATH).git"
    $authOrigin = $origin -replace '^https://', "https://oauth2:$([uri]::EscapeDataString($token))@"
    Invoke-GnarlGit -GitArgs @('remote', 'set-url', 'origin', $authOrigin)
    try {
        if ($OpenPr) {
            Invoke-GnarlGit -GitArgs @('checkout', '-B', $script:GnarlFixBranch)
            Invoke-GnarlGit -GitArgs @('push', '-u', 'origin', $script:GnarlFixBranch, '--force')
            if (-not $env:CI_API_V4_URL -or -not $env:CI_PROJECT_ID) {
                throw 'GitLab MR create requires CI_API_V4_URL and CI_PROJECT_ID.'
            }
            $uri = Get-GnarlGitLabMergeRequestsUri -ApiV4 $env:CI_API_V4_URL -ProjectId $env:CI_PROJECT_ID
            $body = @{
                source_branch = $script:GnarlFixBranch
                target_branch = $DefaultBranch
                title         = 'gnarl auto (review ignores)'
                description   = "Nightly gnarl auto; policy exit $ExitCode (prFrom=$PrFrom)."
            }
            try {
                $created = Invoke-RestMethod -Method Post -Uri $uri -Headers @{ 'PRIVATE-TOKEN' = $token } -Body $body
                Write-GnarlHost "Merge request $($created.iid) : $script:GnarlFixBranch -> $DefaultBranch (policy exit $ExitCode, prFrom=$PrFrom)"
                if ($created.web_url) {
                    Write-GnarlHost $created.web_url
                }
            }
            catch {
                Write-GnarlHost "GitLab merge request create failed (branch $script:GnarlFixBranch pushed targeting $DefaultBranch; an open MR may already exist): $($_.Exception.Message)"
            }
        }
        else {
            Invoke-GnarlGit -GitArgs @('push', 'origin', "HEAD:$DefaultBranch")
            Write-GnarlHost "Pushed to $DefaultBranch (policy exit $ExitCode, prFrom=$PrFrom; no MR)."
        }
    }
    finally {
        Invoke-GnarlGit -GitArgs @('remote', 'set-url', 'origin', $origin)
    }
}

function Invoke-GnarlAutoAndPublish {
    param(
        [Parameter(Mandatory)][string] $GnarlExe,
        [Parameter(Mandatory)][string] $YarnWorkingDirectory,
        [Parameter(Mandatory)][string] $PrFrom,
        [Parameter(Mandatory)][string] $DefaultBranch,
        [Parameter(Mandatory)][string] $RepoRoot,
        [Parameter(Mandatory)][string] $GitHost
    )

    $yarnDir = $YarnWorkingDirectory
    if (-not [System.IO.Path]::IsPathRooted($yarnDir)) {
        $yarnDir = Join-Path $RepoRoot $YarnWorkingDirectory
    }

    Push-Location $yarnDir
    try {
        $code = Invoke-GnarlLogged -GnarlExe $GnarlExe -WorkingDirectory $yarnDir
    }
    finally {
        Pop-Location
    }

    if ($code -eq 1) {
        throw 'gnarl exited 1 (tool error).'
    }

    Push-Location $RepoRoot
    try {
        Switch-GnarlToDefaultBranch -DefaultBranch $DefaultBranch

        $relPaths = Get-GnarlScopedPaths -YarnWorkingDirectory $YarnWorkingDirectory
        foreach ($p in $relPaths) {
            if (Test-Path -LiteralPath (Join-Path $RepoRoot $p)) {
                Invoke-GnarlGit -GitArgs @('add', '--', $p)
            }
        }

        $porcelain = & git status --porcelain -- @relPaths
        if (-not $porcelain) {
            Write-GnarlHost "Working tree clean for gnarl outputs; nothing to commit (policy exit $code, prFrom=$PrFrom)."
            return
        }

        Invoke-GnarlGit -GitArgs @('config', 'user.email', 'gnarl-bot@users.noreply.github.com')
        Invoke-GnarlGit -GitArgs @('config', 'user.name', 'gnarl bot')
        Invoke-GnarlGit -GitArgs @('commit', '-m', 'chore: gnarl auto')

        $openPr = Test-GnarlShouldOpenPullRequest -PrFrom $PrFrom -ExitCode $code
        if ($GitHost -eq 'GitLab') {
            Publish-GnarlGitLab -OpenPr $openPr -DefaultBranch $DefaultBranch -ExitCode $code -PrFrom $PrFrom
        }
        else {
            Publish-GnarlAzureDevOps -OpenPr $openPr -DefaultBranch $DefaultBranch -ExitCode $code -PrFrom $PrFrom
        }
    }
    finally {
        Pop-Location
    }
}

function Invoke-GnarlNightlyMain {
    if ($Mode -eq 'EnsureBinary') {
        if (-not $CacheDirectory) {
            throw 'CacheDirectory is required for EnsureBinary.'
        }
        $path = Install-GnarlWindowsRelease -Version $GnarlVersion -CacheDirectory $CacheDirectory
        Write-Host "gnarl at $path"
        return
    }

    if (-not $GnarlExe) {
        throw 'GnarlExe is required for Auto.'
    }
    $root = $RepoRoot
    if (-not $root) {
        $root = (Get-Location).Path
    }
    Invoke-GnarlAutoAndPublish -GnarlExe $GnarlExe -YarnWorkingDirectory $YarnWorkingDirectory -PrFrom $PrFrom -DefaultBranch $DefaultBranch -RepoRoot $root -GitHost $GitHost
}

$dotSourcedWithoutArgs = $MyInvocation.InvocationName -eq '.' -and $PSBoundParameters.Count -eq 0
if (-not $dotSourcedWithoutArgs) {
    Invoke-GnarlNightlyMain
}
