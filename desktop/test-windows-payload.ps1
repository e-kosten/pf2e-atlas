# Exercise the installer transaction without touching HKCU or an installed app.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Join-Path ([IO.Path]::GetTempPath()) ('atlas-payload-test-' + [guid]::NewGuid().ToString('N'))
$installed = (New-Item -ItemType Directory (Join-Path $root 'installed')).FullName
$payload = (New-Item -ItemType Directory (Join-Path $root 'payload')).FullName
$transaction = Join-Path $PSScriptRoot 'src-tauri/install-atlas-payload.ps1'
$runtimeNames = @('msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')
$coreNames = @('atlas.exe', 'atlas-desktop.exe', 'uninstall.exe', 'THIRD-PARTY-NOTICES.md')
$receipt = 'atlas-desktop-windows-runtime.json'
$names = $coreNames + $runtimeNames + @($receipt)
$registered = $null
# Exercise the installed script without touching the real HKCU.
function Get-ItemProperty {
    param($LiteralPath, $ErrorAction)
    if ($LiteralPath -ne 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\PF2e Atlas') { throw "Unexpected key $LiteralPath" }
    if ($registered) {
        return @{ MainBinaryName = 'atlas-desktop.exe'; DisplayName = 'PF2e Atlas'
            InstallLocation = '"' + $registered + '"'; UninstallString = '"' + (Join-Path $registered 'uninstall.exe') + '"' }
    }
}
function Write-Package([string]$version) {
    $hashes = [ordered]@{}
    foreach ($name in ($coreNames + $runtimeNames)) {
        $path = Join-Path $payload $name
        [IO.File]::WriteAllText($path, "$version-$name")
        if ($runtimeNames -contains $name) { $hashes[$name] = (Get-FileHash -LiteralPath $path).Hash }
    }
    [IO.File]::WriteAllText((Join-Path $payload $receipt), (@{format='atlas-windows-runtime-v1'; files=$hashes} | ConvertTo-Json))
}
function Expect-Refusal([string]$location, [string]$reason) {
    $detail = ''
    try { & $transaction -Payload $payload -InstallDirectory $location } catch { $detail = $_.Exception.Message }
    if (!$detail.Contains($reason)) { throw "Expected refusal naming '$reason', got '$detail'" }
}
function Check-Package([string]$prefix) {
    foreach ($name in ($coreNames + $runtimeNames)) {
        if ([IO.File]::ReadAllText((Join-Path $installed $name)) -ne "$prefix-$name") { throw "Incorrect $name after $prefix package operation." }
    }
}
try {
    Write-Package 'old'
    $isolated = (New-Item -ItemType Directory (Join-Path $root 'other app')).FullName
    $registered = $installed
    foreach ($name in 'vcruntime140.dll', 'uninstall.exe', 'atlas.exe') {
        $foreign = Join-Path $isolated $name
        [IO.File]::Copy((Join-Path $payload $name), $foreign)
        Expect-Refusal $isolated $foreign
        if (@(Get-ChildItem -Force -LiteralPath $isolated).Count -ne 1 -or [IO.File]::ReadAllText($foreign) -ne "old-$name") {
            throw 'Installer changed an unrelated destination.'
        }
        [IO.File]::Delete($foreign)
    }
    # Model an entry arriving after the destination snapshot. Use a fixed
    # inventory boundary so this regression is independent of timing and OS.
    & {
        $lateFile = Join-Path $isolated 'vcruntime140_1.dll'
        [IO.File]::WriteAllText($lateFile, 'late arrival')
        function Get-ChildItem {
            param($LiteralPath, [switch]$Force, $Filter)
            $entries = Microsoft.PowerShell.Management\Get-ChildItem @PSBoundParameters
            if ($LiteralPath -eq $isolated -and !$Filter) {
                return $entries | Where-Object FullName -ne $lateFile
            }
            $entries
        }
        Expect-Refusal $isolated $lateFile
        $remaining = @(Microsoft.PowerShell.Management\Get-ChildItem -LiteralPath $isolated -Force)
        if ($remaining.Count -ne 1 -or [IO.File]::ReadAllText($lateFile) -ne 'late arrival') {
            throw 'Rollback changed a file that appeared after the inventory or left partial Atlas files.'
        }
        [IO.File]::Delete($lateFile)
    }
    $foreign = Join-Path $isolated 'atlas-windows-runtime.json'
    [IO.File]::Copy((Join-Path $payload $receipt), $foreign)
    Expect-Refusal $isolated 'standalone Atlas CLI'
    $registered = $null
    $unrelated = Join-Path $installed 'campaign.txt'
    [IO.File]::WriteAllText($unrelated, 'campaign')
    & $transaction -Payload $payload -InstallDirectory $installed
    Check-Package old
    $journal = Join-Path $installed '.atlas-payload.pending'
    if (!(Test-Path -LiteralPath $journal)) { throw 'Fresh installation retired recovery before registration' }
    # No registry entry yet. Retry the first installation after a process loss.
    & $transaction -Payload $payload -InstallDirectory $installed
    $registered = $installed
    & $transaction -Finish -InstallDirectory $installed
    if (Test-Path -LiteralPath $journal) { throw 'Registration did not retire the journal' }

    Write-Package 'new'
    if ($env:OS -eq 'Windows_NT') {
        $lock = [IO.File]::OpenRead((Join-Path $installed 'atlas.exe'))
        try { Expect-Refusal $installed 'terminal sessions'; Check-Package old } finally { $lock.Dispose() }
    }
    $changed = Join-Path $installed 'msvcp140.dll'
    [IO.File]::WriteAllText($changed, 'unrelated')
    Expect-Refusal $installed $changed
    if ([IO.File]::ReadAllText($changed) -ne 'unrelated') { throw 'Modified runtime lost' }
    [IO.File]::WriteAllText($changed, 'old-msvcp140.dll')

    # Simulate process loss between two per-file replacements in a registered upgrade.
    $stageName = '.atlas-install-' + [guid]::NewGuid().ToString('N')
    $stage = Join-Path $installed $stageName
    [void][IO.Directory]::CreateDirectory("$stage/incoming")
    [void][IO.Directory]::CreateDirectory("$stage/previous")
    $intent = @(foreach ($name in $names) {
        [IO.File]::Copy((Join-Path $payload $name), "$stage/incoming/$name")
        @{ name = $name; existed = $true; sha256 = (Get-FileHash -LiteralPath "$stage/incoming/$name").Hash }
    })
    [IO.File]::WriteAllText($journal, (@{ stage = $stageName; files = $intent } | ConvertTo-Json -Depth 3))
    [IO.File]::Move("$installed/atlas-desktop.exe", "$stage/previous/atlas-desktop.exe")
    [IO.File]::Move("$stage/incoming/atlas-desktop.exe", "$installed/atlas-desktop.exe")
    # One backup was moved before its incoming file was published. A foreign
    # file also replaced the already published host before recovery.
    [IO.File]::Move("$installed/atlas.exe", "$stage/previous/atlas.exe")
    [IO.File]::WriteAllText("$installed/atlas-desktop.exe", 'user replacement')
    Expect-Refusal $installed "$installed$([IO.Path]::DirectorySeparatorChar)atlas-desktop.exe"
    if ([IO.File]::ReadAllText("$installed/atlas-desktop.exe") -ne 'user replacement' -or
        [IO.File]::ReadAllText("$stage/previous/atlas-desktop.exe") -ne 'old-atlas-desktop.exe' -or
        !(Test-Path -LiteralPath $journal)) { throw 'Interrupted rollback discarded a modified file or its backup' }
    [IO.File]::Delete("$installed/atlas-desktop.exe")
    & $transaction -Payload $payload -InstallDirectory $installed
    Check-Package new
    if (Test-Path -LiteralPath $journal) { throw 'Upgrade left a pending payload' }

    [IO.File]::Delete((Join-Path $installed $receipt))
    Expect-Refusal $installed 'does not match the previous Atlas package'
    foreach ($name in $runtimeNames) { [IO.File]::Delete((Join-Path $installed $name)) }
    & $transaction -Payload $payload -InstallDirectory $installed
    Check-Package new
    [IO.File]::WriteAllText($changed, 'later replacement')
    & $transaction -RemoveRuntime -InstallDirectory $installed
    if ([IO.File]::ReadAllText($changed) -ne 'later replacement' -or [IO.File]::ReadAllText($unrelated) -ne 'campaign') {
        throw 'Uninstall changed unrelated files'
    }
    foreach ($name in ($runtimeNames + @($receipt))) {
        if ($name -ne 'msvcp140.dll' -and (Test-Path -LiteralPath (Join-Path $installed $name))) { throw "Uninstall left $name" }
    }
    'Desktop destination ownership, runtime upgrade, crash recovery, and uninstall passed.'
} finally {
    $base = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd([IO.Path]::DirectorySeparatorChar)
    $owned = (Resolve-Path -LiteralPath $root).ProviderPath
    if ([IO.Path]::GetDirectoryName($owned) -ne $base -or [IO.Path]::GetFileName($owned) -notmatch '^atlas-payload-test-[a-f0-9]{32}$') {
        throw "Unexpected test directory $owned"
    }
    Remove-Item -LiteralPath $owned -Recurse -Force
}
