[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$QualificationRoot,
    [Parameter(Mandatory)][string]$InstallDirectory,
    [switch]$Restart
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $QualificationRoot).Path
if ((Get-Content -LiteralPath (Join-Path $root 'qualification-root.txt') -Raw).Trim() -ne $root) {
    throw 'The disposable qualification root marker is missing.'
}
$request = Get-Content -LiteralPath (Join-Path $root 'active-case.json') -Raw | ConvertFrom-Json
$caseRoot = (Resolve-Path -LiteralPath $request.caseRoot).Path
trap {
    $_ | Out-String | Set-Content -LiteralPath (Join-Path $caseRoot 'hook-error.txt') -Encoding utf8
    exit 1
}
if (!$caseRoot.StartsWith($root + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'The qualification case is outside the disposable root.'
}
$installed = (Resolve-Path -LiteralPath $InstallDirectory).Path
if ($installed -ne $request.installDirectory) { throw 'Unexpected qualification installation.' }
if ($Restart) {
    # The fixture installer skips RunAsUser, which may discard caller environment.
    # Wait for that installer to exit, then launch the genuine installed target
    # with explicit fixture paths. Normal published installers are unaffected.
    $parentId = (Get-CimInstance Win32_Process -Filter "ProcessId=$PID").ParentProcessId
    $parent = Get-Process -Id $parentId -ErrorAction SilentlyContinue
    if ($parent -and !$parent.WaitForExit(30000)) { throw 'The fixture installer did not exit.' }
    $env:HOME = $request.home
    $env:USERPROFILE = $request.home
    $env:LOCALAPPDATA = $request.home
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$($request.cdpPort)"
    $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $caseRoot 'webview2'
    Remove-Item Env:ATLAS_PILOT_SMOKE, Env:ATLAS_PILOT_HOME -ErrorAction SilentlyContinue
    $target = Start-Process -FilePath (Join-Path $installed 'atlas-desktop.exe') -WorkingDirectory $request.home -PassThru
    @{ pid = $target.Id; home = $request.home; cdpPort = $request.cdpPort; fixtureControlledRestart = $true } |
        ConvertTo-Json | Set-Content -LiteralPath (Join-Path $caseRoot 'restart.json') -Encoding utf8
    exit 0
}

# POSTINSTALL runs after NSIS has replaced both executables and before its /R
# restart. This is fixture scheduling in a disposable installer, not an app hook.
$old = @(Get-CimInstance Win32_Process | Where-Object {
    $_.ExecutablePath -and
    ([IO.Path]::GetDirectoryName($_.ExecutablePath) -eq $installed) -and
    ($_.Name -in @('atlas-desktop.exe', 'atlas.exe'))
})
if ($old.Count -ne 0) { throw 'The previous host or bundled CLI is still running at replacement.' }
$hostHash = (Get-FileHash -LiteralPath (Join-Path $installed 'atlas-desktop.exe')).Hash.ToLowerInvariant()
$cliHash = (Get-FileHash -LiteralPath (Join-Path $installed 'atlas.exe')).Hash.ToLowerInvariant()
if ($hostHash -ne $request.targetHostSha256 -or $cliHash -ne $request.targetCliSha256) {
    throw 'NSIS did not install the exact target host and CLI.'
}
$selection = Join-Path $request.home 'pf2e-atlas/workspace-selection.json'
$config = Get-Content -LiteralPath $selection -Raw | ConvertFrom-Json
if ($config.active -ne 'original' -or !$config.pending -or $config.pending.target.version -ne $request.targetVersion) {
    throw 'The reviewed selection was not kept pending during replacement.'
}
$resolvedDatabase = Resolve-Path -LiteralPath $config.pending.admission.workspace
$database = $resolvedDatabase.ProviderPath
$checkedDatabase = $database
if ($checkedDatabase.StartsWith('\\?\')) { $checkedDatabase = $checkedDatabase.Substring(4) }
@{
    admissionPath = $config.pending.admission.workspace
    powershellPath = $resolvedDatabase.Path
    providerPath = $resolvedDatabase.ProviderPath
    normalizedPath = $checkedDatabase
    caseRoot = $caseRoot
} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $caseRoot 'path-check.json') -Encoding utf8
if (!$checkedDatabase.StartsWith($caseRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'The reviewed database is outside the disposable case.'
}
$database = $checkedDatabase
if ($request.mode -notin @('normal', 'identity', 'ledger')) { throw 'Unknown qualification mode.' }
if ($request.mode -ne 'normal') {
    $code = @'
import os, shutil, sqlite3, sys, uuid
path, mode = sys.argv[1:]
db = sqlite3.connect(path)
db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
db.close()
if mode == 'identity':
    replacement = path + '.replacement'
    source, target = sqlite3.connect(path), sqlite3.connect(replacement)
    source.backup(target)
    target.execute('UPDATE workspace SET workspace_key=? WHERE id=1', (uuid.uuid4().hex,))
    target.commit()
    source.close()
    target.close()
    shutil.copy2(path, path + '.reviewed')
    os.replace(replacement, path)
elif mode == 'ledger':
    db = sqlite3.connect(path)
    changed = db.execute("UPDATE schema_migrations SET checksum=? WHERE version=(SELECT max(version) FROM schema_migrations)", ('0' * 64,))
    assert changed.rowcount == 1
    db.commit()
    db.execute('PRAGMA wal_checkpoint(TRUNCATE)')
    db.close()
'@
    # The fixture mutation intentionally happens after admission is released.
    & python -c $code $database $request.mode
    if ($LASTEXITCODE -ne 0) { throw 'Could not mutate the disposable race fixture.' }
}
@{
    mode = $request.mode
    oldProcesses = $old.Count
    hostSha256 = $hostHash
    cliSha256 = $cliHash
    database = $database
    databaseSha256 = (Get-FileHash -LiteralPath $database).Hash.ToLowerInvariant()
    selectionSha256 = (Get-FileHash -LiteralPath $selection).Hash.ToLowerInvariant()
    pending = $config.pending
} | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath (Join-Path $caseRoot 'postinstall.json') -Encoding utf8
