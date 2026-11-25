# Exercise the installer transaction without touching HKCU or an installed app.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Join-Path ([IO.Path]::GetTempPath()) ('atlas-payload-test-' + [guid]::NewGuid().ToString('N'))
$installed = (New-Item -ItemType Directory (Join-Path $root 'installed')).FullName
$payload = (New-Item -ItemType Directory (Join-Path $root 'payload')).FullName
$transaction = Join-Path $PSScriptRoot 'src-tauri/install-atlas-payload.ps1'
$names = @('atlas.exe', 'atlas-desktop.exe', 'vcruntime140.dll')
function Check-Package([string]$prefix) {
    foreach ($name in $names) {
        if ([IO.File]::ReadAllText((Join-Path $installed $name)) -ne "$prefix-$name") { throw "Incorrect $name after $prefix package operation." }
    }
}
try {
    foreach ($name in $names) {
        [IO.File]::WriteAllText((Join-Path $installed $name), "old-$name")
        [IO.File]::WriteAllText((Join-Path $payload $name), "new-$name")
    }
    $lock = [IO.File]::OpenRead((Join-Path $installed 'atlas.exe'))
    try {
        $errorMessage = ''
        try { & $transaction -Payload $payload -InstallDirectory $installed } catch { $errorMessage = $_.Exception.Message }
        if ($errorMessage -notmatch 'terminal sessions') { throw "Expected a locked-file refusal, got: $errorMessage" }
        Check-Package 'old'
    } finally { $lock.Dispose() }

    # Model a process loss after one old file was saved and its new bytes
    # installed. Retry must restore the old file before publishing the complete set.
    $stageName = '.atlas-install-' + [guid]::NewGuid().ToString('N')
    $stage = Join-Path $installed $stageName
    [void][IO.Directory]::CreateDirectory("$stage/incoming")
    [void][IO.Directory]::CreateDirectory("$stage/previous")
    $intent = @(foreach ($name in $names) {
        [IO.File]::Copy((Join-Path $payload $name), "$stage/incoming/$name")
        @{ name = $name; existed = $true; sha256 = (Get-FileHash -LiteralPath "$stage/incoming/$name").Hash }
    })
    $journal = Join-Path $installed '.atlas-payload.pending'
    [IO.File]::WriteAllText($journal, (@{ stage = $stageName; files = $intent } | ConvertTo-Json -Depth 3))
    [IO.File]::Move("$installed/atlas-desktop.exe", "$stage/previous/atlas-desktop.exe")
    [IO.File]::Move("$stage/incoming/atlas-desktop.exe", "$installed/atlas-desktop.exe")
    & $transaction -Payload $payload -InstallDirectory $installed
    Check-Package 'new'
    if (Test-Path -LiteralPath $journal) { throw 'Retry left a pending payload.' }
    'Locked-file preservation, interrupted replacement and retry passed.'
} finally {
    $base = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')
    $owned = (Resolve-Path -LiteralPath $root).ProviderPath
    if ([IO.Path]::GetDirectoryName($owned) -ne $base -or [IO.Path]::GetFileName($owned) -notmatch '^atlas-payload-test-[a-f0-9]{32}$') {
        throw "Unexpected test directory $owned"
    }
    Remove-Item -LiteralPath $owned -Recurse -Force
}
