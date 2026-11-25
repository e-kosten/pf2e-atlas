# Runs under Windows PowerShell 5.1 in the per-user NSIS installer.
[CmdletBinding(DefaultParameterSetName='Install')]
param(
    [Parameter(Mandatory,ParameterSetName='Install')][string]$Payload,
    [Parameter(Mandatory,ParameterSetName='Finish')][switch]$Finish,
    [Parameter(Mandatory,ParameterSetName='Remove')][switch]$RemoveRuntime,
    [Parameter(Mandatory)][string]$InstallDirectory
)

$ErrorActionPreference = 'Stop'
$destination = (Resolve-Path -LiteralPath $InstallDirectory).ProviderPath
$receiptName = 'atlas-desktop-windows-runtime.json'
# Installed CRTs from the pinned predecessor in qualify-windows.ps1. The
# qualification records the packaged digests in Actions run 45681581959.
$legacyHashes = @{
    'msvcp140.dll' = '0f885b509a685d2bbfa652fed26b5fb31d88fbdab0a978c641d1c7b8aa460aa9'
    'msvcp140_1.dll' = 'bfad5aef4c63a669e3c140655cdfdf395b6c979b400a447bd5dcb65ed8826c3d'
    'vcruntime140.dll' = 'd5e4d9a3e835fa679450145d6a7d94e36573a509317111904d9b3712c30d9066'
    'vcruntime140_1.dll' = '1f2d41c4aa5db0bc33ebf7b66d72943a817d7ce6cbe880502a9403823633093f'
}
$coreNames = @('atlas.exe', 'atlas-desktop.exe', 'THIRD-PARTY-NOTICES.md', 'uninstall.exe')
$names = $coreNames + @($legacyHashes.Keys) + @($receiptName)

function Test-RegisteredDestination {
    $key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\PF2e Atlas'
    $registration = Get-ItemProperty -LiteralPath $key -ErrorAction SilentlyContinue
    if ($registration.DisplayName -ne 'PF2e Atlas' -or $registration.MainBinaryName -ne 'atlas-desktop.exe' -or
        !$registration.InstallLocation -or !$registration.UninstallString) { return $false }
    $registered = [IO.Path]::GetFullPath($registration.InstallLocation.Trim('"')).TrimEnd([IO.Path]::DirectorySeparatorChar)
    return $registered -eq $destination.TrimEnd([IO.Path]::DirectorySeparatorChar) -and
        [IO.Path]::GetFullPath($registration.UninstallString.Trim('"')) -eq (Join-Path $destination 'uninstall.exe')
}
function Read-Receipt([string]$directory) {
    $path = Join-Path $directory $receiptName
    if ([IO.File]::GetAttributes($path) -band [IO.FileAttributes]::ReparsePoint) { throw "Invalid desktop receipt: $path" }
    $record = [IO.File]::ReadAllText($path) | ConvertFrom-Json
    if ($record.format -cne 'atlas-windows-runtime-v1' -or @($record.files.PSObject.Properties).Count -ne $legacyHashes.Count) {
        throw "Invalid desktop receipt: $path"
    }
    $hashes = @{}
    foreach ($name in $legacyHashes.Keys) {
        if ($record.files.$name -notmatch '^[0-9a-f]{64}$') { throw "Invalid desktop receipt for $name at $path" }
        $hashes[$name] = $record.files.$name
    }
    return $hashes
}

# Writable handles refuse mapped executables and DLLs. Sharing DELETE permits
# renames on this volume while preventing new launches throughout replacement.
$handles = [Collections.Generic.List[IO.FileStream]]::new()
function Get-PayloadHash([string]$path) {
    $hash = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Read, ([IO.FileShare]::ReadWrite -bor [IO.FileShare]::Delete))
    try { return [BitConverter]::ToString($hash.ComputeHash($stream)).Replace('-', '') }
    finally { $stream.Dispose(); $hash.Dispose() }
}
function Hold-File([string]$path) {
    if ([IO.File]::GetAttributes($path) -band [IO.FileAttributes]::ReparsePoint) {
        throw "The installation contains a linked file at $path."
    }
    try {
        $share = [IO.FileShare]::Read -bor [IO.FileShare]::Delete
        $handles.Add([IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Write, $share))
    } catch {
        throw "Close Atlas and any terminal sessions using $path, check its permissions, then retry."
    }
}
function Release-Files {
    foreach ($handle in $handles) { $handle.Dispose() }
    $handles.Clear()
}

# The intent is published before the first move and retired after the last.
# Each file's location records progress, including an interrupted rollback.
$marker = Join-Path $destination '.atlas-payload.pending'
function Restore-Payload($intent, [string]$stage) {
    foreach ($entry in $intent) {
        $name = $entry.name
        $slot = Join-Path $destination $name
        $old = Join-Path "$stage/previous" $name
        $new = Join-Path "$stage/incoming" $name
        if (![IO.File]::Exists($new) -and ([IO.File]::Exists($old) -or !$entry.existed)) {
            [IO.File]::Move($slot, $new)
        }
        if ([IO.File]::Exists($old)) { [IO.File]::Move($old, $slot) }
    }
}
function Remove-Stage([string]$stage, $names) {
    # The marker has been retired and the installed set is consistent. Cleanup
    # can be retried separately if a scanner opens a backup without delete sharing.
    try {
        foreach ($directory in "$stage/incoming", "$stage/previous") {
            foreach ($name in $names) { [IO.File]::Delete((Join-Path $directory $name)) }
            [IO.Directory]::Delete($directory)
        }
        [IO.File]::Delete((Join-Path $stage 'intent.json'))
        [IO.Directory]::Delete($stage)
    } catch {
        Write-Warning "The installed files are consistent. Temporary files remain at $stage. $($_.Exception.Message)"
    }
}
function Recover-Payload([bool]$completeFirstInstall) {
    if (!(Test-Path -LiteralPath $marker)) { return }
    if ([IO.File]::GetAttributes($marker) -band [IO.FileAttributes]::ReparsePoint) { throw "Invalid journal: $marker" }
    $saved = [IO.File]::ReadAllText($marker) | ConvertFrom-Json
    if ($saved.stage -notmatch '^\.atlas-install-[a-f0-9]{32}$') { throw "Invalid journal: $marker" }
    $stage = Join-Path $destination $saved.stage
    foreach ($directory in $stage, "$stage/incoming", "$stage/previous") {
        if ([IO.File]::GetAttributes($directory) -band [IO.FileAttributes]::ReparsePoint) { throw "Invalid stage: $directory" }
    }
    $intent = @($saved.files)
    if ($intent.Count -eq 0 -or (@($intent.name | Sort-Object -Unique)).Count -ne $intent.Count) { throw "Invalid journal: $marker" }
    foreach ($entry in $intent) {
        if ($names -notcontains $entry.name -or $entry.existed -isnot [bool] -or $entry.sha256 -notmatch '^[0-9a-f]{64}$') {
            throw "The interrupted installation at $stage requires its original installer."
        }
        $slot = Join-Path $destination $entry.name
        $old = Join-Path "$stage/previous" $entry.name
        $new = Join-Path "$stage/incoming" $entry.name
        if (![IO.File]::Exists($new) -and ([IO.File]::Exists($old) -or !$entry.existed)) {
            if ((Get-PayloadHash $slot) -ne $entry.sha256) { throw "The interrupted installation's file changed: $slot" }
        }
        foreach ($path in $slot, $old) { if ([IO.File]::Exists($path)) { Hold-File $path } }
    }
    if ($completeFirstInstall) {
        if (!$saved.fresh -or $intent.Count -ne $names.Count) { throw "Unexpected unfinished installation at $stage" }
        foreach ($entry in $intent) {
            if ($entry.existed -or [IO.File]::Exists((Join-Path "$stage/incoming" $entry.name))) {
                throw "Incomplete first installation at $stage. Rerun setup."
            }
        }
    } else {
        Restore-Payload $intent $stage
    }
    [IO.File]::Delete($marker)
    Release-Files
    Remove-Stage $stage $intent.name
}

$lock = [IO.FileStream]::new((Join-Path $destination '.atlas-payload.lock'), [IO.FileMode]::CreateNew,
    [IO.FileAccess]::ReadWrite, [IO.FileShare]::None, 4096, [IO.FileOptions]::DeleteOnClose)
try {
    foreach ($name in '.atlas-cli-install.lock', '.atlas-cli-installing') {
        if (Get-ChildItem -LiteralPath $destination -Force -Filter $name) {
            throw "A standalone CLI installation is in progress at $destination. Finish it and choose a dedicated desktop folder."
        }
    }
    $registered = Test-RegisteredDestination
    if ($Finish -or $RemoveRuntime) {
        if (!$registered) { throw "The registered Atlas desktop installation does not match $destination. Rerun setup in a dedicated folder." }
        Recover-Payload $true
        if ($Finish) { return }
        $receipt = Join-Path $destination $receiptName
        foreach ($name in 'atlas-desktop.exe', 'atlas.exe') {
            $path = Join-Path $destination $name
            if ([IO.File]::Exists($path)) { Hold-File $path }
        }
        if (![IO.File]::Exists($receipt)) { return }
        try { $hashes = Read-Receipt $destination } catch {
            Write-Warning "Could not verify the desktop receipt. Leaving the runtime files in $destination."
            return
        }
        $remove = @(foreach ($name in $hashes.Keys) {
            $path = Join-Path $destination $name
            if (![IO.File]::Exists($path) -or ([IO.File]::GetAttributes($path) -band [IO.FileAttributes]::ReparsePoint)) { continue }
            if ((Get-PayloadHash $path) -ne $hashes[$name]) { continue }
            Hold-File $path
            if ((Get-PayloadHash $path) -eq $hashes[$name]) { $path }
        })
        Hold-File $receipt
        foreach ($path in $remove) { [IO.File]::Delete($path) }
        [IO.File]::Delete($receipt)
        return
    }

    Recover-Payload $false
    $files = @(Get-ChildItem -LiteralPath $Payload -Force)
    if ($files.Count -ne $names.Count) { throw 'The Windows package is incomplete.' }
    foreach ($file in $files) {
        if ($names -notcontains $file.Name -or $file.PSIsContainer -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw "Unexpected installer entry: $($file.Name)"
        }
    }
    $incomingHashes = Read-Receipt $Payload
    $existing = @{}
    foreach ($item in Get-ChildItem -LiteralPath $destination -Force) { $existing[$item.Name] = $item }
    if ($existing.ContainsKey('atlas-windows-runtime.json')) {
        throw "This folder contains the standalone Atlas CLI. Choose a dedicated desktop installation folder."
    }
    $installedHashes = $legacyHashes
    if ($registered -and $existing.ContainsKey($receiptName)) { $installedHashes = Read-Receipt $destination }
    $stageName = '.atlas-install-' + [guid]::NewGuid().ToString('N')
    $stage = Join-Path $destination $stageName
    [void][IO.Directory]::CreateDirectory("$stage/incoming")
    [void][IO.Directory]::CreateDirectory("$stage/previous")
    try {
        $intent = @(foreach ($file in $files) {
            $new = Join-Path "$stage/incoming" $file.Name
            $slot = Join-Path $destination $file.Name
            [IO.File]::Copy($file.FullName, $new)
            $hash = Get-PayloadHash $new
            Hold-File $new
            if ($incomingHashes.ContainsKey($file.Name) -and $hash -ne $incomingHashes[$file.Name]) {
                throw "The packaged runtime does not match its receipt: $($file.Name)"
            }
            if ($existing.ContainsKey($file.Name)) {
                if (!$registered) { throw "An existing file occupies $slot. Choose a dedicated folder for Atlas." }
                Hold-File $slot
                if ($installedHashes.ContainsKey($file.Name) -and (Get-PayloadHash $slot) -ne $installedHashes[$file.Name]) {
                    throw "The existing runtime at $slot does not match the previous Atlas package. Choose a dedicated folder."
                }
            }
            [pscustomobject]@{ name = $file.Name; existed = [IO.File]::Exists($slot); sha256 = $hash }
        })
        $record = @{ stage = $stageName; files = $intent; fresh = !$registered } | ConvertTo-Json -Depth 3
        $prepared = Join-Path $stage 'intent.json'
        [IO.File]::WriteAllText($prepared, $record)
        [IO.File]::Move($prepared, $marker)
        try {
            foreach ($entry in $intent) {
                $slot = Join-Path $destination $entry.name
                if ($entry.existed) { [IO.File]::Move($slot, (Join-Path "$stage/previous" $entry.name)) }
                [IO.File]::Move((Join-Path "$stage/incoming" $entry.name), $slot)
            }
            # The fresh installation remains recoverable until NSIS has registered
            # its destination. A retry before registration rolls back this payload.
            if ($registered) { [IO.File]::Delete($marker) }
        } catch {
            Restore-Payload $intent $stage
            [IO.File]::Delete($marker)
            throw
        }
    } finally {
        Release-Files
        if (![IO.File]::Exists($marker)) { Remove-Stage $stage $files.Name }
    }
} finally {
    Release-Files
    $lock.Dispose()
}
