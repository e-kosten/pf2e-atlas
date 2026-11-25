# Runs under Windows PowerShell 5.1 in the per-user NSIS installer.
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Payload,
    [Parameter(Mandatory)][string]$InstallDirectory
)

$ErrorActionPreference = 'Stop'
$destination = (Resolve-Path -LiteralPath $InstallDirectory).ProviderPath
$source = (Resolve-Path -LiteralPath $Payload).ProviderPath
$files = @(Get-ChildItem -LiteralPath $source -Force)
if ($files.Name -notcontains 'atlas.exe' -or $files.Name -notcontains 'atlas-desktop.exe') {
    throw 'The Windows package is incomplete.'
}
foreach ($file in $files) {
    if ($file.PSIsContainer -or ($file.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Unexpected installer entry: $($file.Name)"
    }
}

# Writable handles refuse mapped executables and DLLs. Sharing DELETE permits
# renames on this volume while preventing new launches throughout replacement.
$handles = [Collections.Generic.List[IO.FileStream]]::new()
function Get-PayloadHash([string]$path) {
    $hash = [Security.Cryptography.SHA256]::Create()
    $stream = [IO.File]::OpenRead($path)
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
function Recover-Payload {
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
        if ($files.Name -notcontains $entry.name -or $entry.existed -isnot [bool] -or $entry.sha256 -notmatch '^[0-9a-f]{64}$') {
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
    Restore-Payload $intent $stage
    [IO.File]::Delete($marker)
    Release-Files
    Remove-Stage $stage $intent.name
}

$lock = [IO.FileStream]::new((Join-Path $destination '.atlas-payload.lock'), [IO.FileMode]::CreateNew,
    [IO.FileAccess]::ReadWrite, [IO.FileShare]::None, 4096, [IO.FileOptions]::DeleteOnClose)
try {
    Recover-Payload
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
            if (Test-Path -LiteralPath $slot) { Hold-File $slot }
            [pscustomobject]@{ name = $file.Name; existed = [IO.File]::Exists($slot); sha256 = $hash }
        })
        $record = @{ stage = $stageName; files = $intent } | ConvertTo-Json -Depth 3
        $prepared = Join-Path $stage 'intent.json'
        [IO.File]::WriteAllText($prepared, $record)
        [IO.File]::Move($prepared, $marker)
        try {
            foreach ($entry in $intent) {
                $slot = Join-Path $destination $entry.name
                if ($entry.existed) { [IO.File]::Move($slot, (Join-Path "$stage/previous" $entry.name)) }
                [IO.File]::Move((Join-Path "$stage/incoming" $entry.name), $slot)
            }
            [IO.File]::Delete($marker)
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
