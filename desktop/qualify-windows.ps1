[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$PackagedCli,
    [Parameter(Mandatory)][string]$ScratchDirectory
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
if ($env:OS -ne 'Windows_NT') { throw 'Run this rehearsal on a disposable Windows x64 runner.' }
if ($env:GITHUB_ACTIONS -ne 'true' -or !$env:RUNNER_TEMP) {
    throw 'This rehearsal installs test packages and is restricted to a disposable GitHub Actions runner.'
}
$repo = Split-Path -Parent $PSScriptRoot
$cli = (Resolve-Path -LiteralPath $PackagedCli).Path
$scratch = (Resolve-Path -LiteralPath $ScratchDirectory).Path
if ($scratch -ne (Resolve-Path -LiteralPath $env:RUNNER_TEMP).Path) { throw 'Use this runner''s temporary directory.' }
if (Test-Path -LiteralPath (Join-Path $env:LOCALAPPDATA 'pf2e-atlas')) {
    throw 'The runner already has Atlas user data. Use a fresh disposable runner.'
}
if (@(Get-CimInstance Win32_Process -Filter "Name='atlas-desktop.exe'").Count -ne 0) {
    throw 'An Atlas app is already running on this runner.'
}
$root = Join-Path $scratch ('atlas-updater-qualification-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
$root | Set-Content -LiteralPath (Join-Path $root 'qualification-root.txt') -Encoding utf8
Write-Output "Disposable Windows updater evidence: $root"
if ($env:GITHUB_OUTPUT) { "evidence=$root" >> $env:GITHUB_OUTPUT }

function Invoke-Checked {
    param([string]$Command, [string[]]$Arguments)
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Command failed with exit code $LASTEXITCODE." }
}
function Escape-Nsis([string]$Value) { return $Value.Replace('$', '$$').Replace('"', '$\"') }
function Get-NsisHostHash([string]$Path) {
    # Tauri 2.12.4 bundles this marker patch, then restores the unpatched build
    # output. Predict the packaged bytes instead of hashing that restored file.
    $bytes = [IO.File]::ReadAllBytes($Path)
    $token = '__TAURI_BUNDLE_TYPE_VAR_UNK'
    $offset = [Text.Encoding]::ASCII.GetString($bytes).IndexOf($token, [StringComparison]::Ordinal)
    if ($offset -lt 0) { throw 'The built host has no Tauri bundle-type marker.' }
    $replacement = [Text.Encoding]::ASCII.GetBytes('__TAURI_BUNDLE_TYPE_VAR_NSS')
    [Array]::Copy($replacement, 0, $bytes, $offset, $replacement.Length)
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($bytes)).ToLowerInvariant()
}
$original = @{}
foreach ($name in @('ATLAS_UPDATER_PUBLIC_KEY', 'ATLAS_UPDATER_REHEARSAL_KEY', 'ATLAS_UPDATER_REHEARSAL_ROOT',
    'ATLAS_UPDATER_REHEARSAL_CA', 'ATLAS_DESKTOP_RELEASE', 'ATLAS_DESKTOP_SOURCE_COMMIT', 'ATLAS_WORKSPACE_SCHEMA',
    'CARGO_TARGET_DIR', 'CARGO_ENCODED_RUSTFLAGS')) { $original[$name] = [Environment]::GetEnvironmentVariable($name) }
Push-Location $repo
try {
    $commit = (& git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $commit -notmatch '^[0-9a-f]{40}$') { throw 'Missing candidate identity.' }
    $stable = '99.0.0'
    $nightly = '99.1.0-nightly.' + [DateTime]::UtcNow.ToString('yyyyMMddHHmmss') + '.g' + $commit.Substring(0, 12)
    $key = Join-Path $root 'disposable.key'
    $signer = Join-Path $PSScriptRoot 'node_modules/@tauri-apps/cli/tauri.js'
    & node $signer signer generate --ci --write-keys $key *> $null
    if ($LASTEXITCODE -ne 0) { throw 'Could not generate the disposable signing key.' }
    $env:ATLAS_UPDATER_PUBLIC_KEY = (Get-Content -LiteralPath "$key.pub" -Raw).Trim()
    $env:ATLAS_UPDATER_REHEARSAL_KEY = $key
    $tlsKey = Join-Path $root 'localhost.key'
    $tlsCert = Join-Path $root 'localhost.crt'
    Invoke-Checked openssl @('req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
        '-keyout', $tlsKey, '-out', $tlsCert, '-subj', '/CN=Atlas disposable qualification',
        '-addext', 'subjectAltName=IP:127.0.0.1', '-addext', 'basicConstraints=critical,CA:TRUE')
    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $port = $listener.LocalEndpoint.Port
    $listener.Stop()
    $env:ATLAS_UPDATER_REHEARSAL_ROOT = "https://127.0.0.1:$port/releases"
    $env:ATLAS_UPDATER_REHEARSAL_CA = Get-Content -LiteralPath $tlsCert -Raw

    # A harmless PE overlay distinguishes the old fixture CLI's bytes without
    # building a second workspace. It must still pass the real CLI probe.
    $oldCli = Join-Path $root 'old-atlas.exe'
    Copy-Item -LiteralPath $cli -Destination $oldCli
    $overlay = [Text.Encoding]::UTF8.GetBytes("`r`nAtlas disposable old-sidecar qualification fixture`r`n")
    $stream = [IO.File]::Open($oldCli, [IO.FileMode]::Append)
    try { $stream.Write($overlay, 0, $overlay.Length) } finally { $stream.Dispose() }
    Invoke-Checked $oldCli @('desktop-probe', '--update-compatibility')
    $oldInstaller = Join-Path $root 'old-setup.exe'
    & "$PSScriptRoot/bundle-windows.ps1" -PackagedCli $oldCli -Version $stable -OutputInstaller $oldInstaller -HostTargetDirectory "$PSScriptRoot/src-tauri/target"
    if ($LASTEXITCODE -ne 0) { throw 'Could not build the old rehearsal installer.' }
    $oldHostHash = Get-NsisHostHash "$PSScriptRoot/src-tauri/target/x86_64-pc-windows-msvc/release/atlas-desktop.exe"
    $targetInstaller = Join-Path $root 'atlas-desktop-x86_64-pc-windows-msvc-setup.exe'
    & "$PSScriptRoot/bundle-windows.ps1" -PackagedCli $cli -Version $nightly -OutputInstaller $targetInstaller -HostTargetDirectory "$PSScriptRoot/src-tauri/target"
    if ($LASTEXITCODE -ne 0) { throw 'Could not build the target rehearsal installer.' }

    # Add an installer-only fixture hook after the ordinary packaging path has
    # prepared CRTs, sidecar, notices and compatibility metadata.
    $hooks = Join-Path $root 'postinstall.nsh'
    $macro = @'
!macro NSIS_HOOK_POSTINSTALL
  ExecWait '"%PWSH%" -NoProfile -File "%HOOK%" -QualificationRoot "%ROOT%" -InstallDirectory "$INSTDIR"' $0
  StrCmp $0 0 +2
  Abort "Atlas qualification POSTINSTALL hook failed"
  Exec '"%PWSH%" -NoProfile -File "%HOOK%" -QualificationRoot "%ROOT%" -InstallDirectory "$INSTDIR" -Restart'
  Quit
!macroend
'@
    $macro = $macro.Replace('%PWSH%', (Escape-Nsis (Get-Command pwsh).Source)).Replace('%HOOK%', (Escape-Nsis "$PSScriptRoot/qualify-windows.hook.ps1")).Replace('%ROOT%', (Escape-Nsis $root))
    $macro | Set-Content -LiteralPath $hooks -Encoding utf8
    $config = Join-Path $root 'target-config.json'
    @{ version = $nightly; bundle = @{ windows = @{ nsis = @{ installerHooks = $hooks } } } } |
        ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $config -Encoding utf8
    $env:ATLAS_DESKTOP_RELEASE = $nightly
    $env:ATLAS_DESKTOP_SOURCE_COMMIT = $commit
    $compatibility = Get-Content -LiteralPath "$targetInstaller.compatibility.json" -Raw | ConvertFrom-Json
    $env:ATLAS_WORKSPACE_SCHEMA = [string]$compatibility.workspaceSchema
    $env:CARGO_TARGET_DIR = "$PSScriptRoot/src-tauri/target"
    $env:CARGO_ENCODED_RUSTFLAGS = '-Ctarget-feature=+crt-static'
    Push-Location $PSScriptRoot
    try { Invoke-Checked node @('./node_modules/@tauri-apps/cli/tauri.js', 'build', '--target', 'x86_64-pc-windows-msvc', '--bundles', 'nsis', '--no-sign', '--config', $config, '--', '--locked') }
    finally { Pop-Location }
    Copy-Item -LiteralPath "$PSScriptRoot/src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/PF2e Atlas_${nightly}_x64-setup.exe" -Destination $targetInstaller -Force
    Invoke-Checked node @('desktop/updater-package.mjs', 'sign', "v$nightly", $targetInstaller)
    $targetHostHash = Get-NsisHostHash "$PSScriptRoot/src-tauri/target/x86_64-pc-windows-msvc/release/atlas-desktop.exe"
    @{
        root = $root; oldInstaller = $oldInstaller; installer = $targetInstaller; stable = $stable; nightly = $nightly
        commit = $commit; port = $port; tlsKey = $tlsKey; tlsCert = $tlsCert
        oldHostSha256 = $oldHostHash; oldCliSha256 = (Get-FileHash $oldCli).Hash.ToLowerInvariant()
        targetHostSha256 = $targetHostHash; targetCliSha256 = (Get-FileHash $cli).Hash.ToLowerInvariant()
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'qualification.json') -Encoding utf8
    Invoke-Checked node @('desktop/qualify-windows.mjs', (Join-Path $root 'qualification.json'))
}
finally {
    Pop-Location
    foreach ($name in $original.Keys) { [Environment]::SetEnvironmentVariable($name, $original[$name]) }
}
