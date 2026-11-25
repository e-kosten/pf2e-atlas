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
function Get-NsisRuntimePath([string]$Path) {
    # NSIS expands $ variables in installer instructions. A preprocessor
    # !include takes a literal native filename and does not use this escaping.
    return (Get-Item -LiteralPath $Path).FullName.Replace('$', '$$')
}
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
    Invoke-Checked powershell.exe @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "$PSScriptRoot/test-windows-payload.ps1")
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
        '-addext', 'subjectAltName=IP:127.0.0.1', '-addext', 'basicConstraints=critical,CA:FALSE',
        '-addext', 'extendedKeyUsage=serverAuth', '-addext', 'keyUsage=critical,digitalSignature,keyEncipherment')
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

    # Keep the ordinary candidate for the direct upgrade from a published
    # predecessor. That host predates the draft guard and uses its original CLI.
    $directInstaller = Join-Path $root 'direct-target.exe'
    Copy-Item -LiteralPath $targetInstaller -Destination $directInstaller
    $directHostHash = Get-NsisHostHash "$PSScriptRoot/src-tauri/target/x86_64-pc-windows-msvc/release/atlas-desktop.exe"
    $priorTag = 'nightly/20261123213244.g4e2fbbf86d52'
    $priorHash = '342d207c9bfcba0766e0660fc0aaf870e42470e28beb1b7223bd9710da49d3a1'
    $priorInstaller = Join-Path $root 'published-predecessor.exe'
    Invoke-WebRequest "https://github.com/e-kosten/pf2e-atlas/releases/download/$priorTag/atlas-desktop-x86_64-pc-windows-msvc-setup.exe" -OutFile $priorInstaller
    if ((Get-FileHash -LiteralPath $priorInstaller).Hash -ne $priorHash) { throw 'Published predecessor checksum mismatch.' }

    # Add an installer-only fixture hook after the ordinary packaging path has
    # prepared CRTs, sidecar, notices and compatibility metadata.
    $hooks = Join-Path $root 'postinstall.nsh'
    $macro = @'
!include "%ATLAS_INSTALLER_HOOK%"
; The fixture invokes the real helper, then starts a competing process at the
; exact gap between the helper returning and NSIS publishing/removing metadata.
!macro QualifyAtlasPayload Arguments
  !insertmacro AtlasPayload '${Arguments}'
  ${If} ${FileExists} "%ROOT%\qualify-lifecycle"
    !if '${Arguments}' != '-Finish'
      !if '${Arguments}' == '-RemoveRuntime'
        ExecWait '"%DIRECT%" /S /UPDATE /D=$INSTDIR-other' $0
        StrCpy $1 "%ROOT%\uninstall-refused-setup"
      !else
        ExecWait '"$INSTDIR\uninstall.exe" /S _?=$INSTDIR' $0
        StrCpy $1 "%ROOT%\setup-refused-uninstall"
      !endif
      ${If} $0 != 1618
        SetErrorLevel 2
        Quit
      ${EndIf}
      FileOpen $0 "$1" w
      FileWrite $0 "1618"
      FileClose $0
    !endif
  ${EndIf}
!macroend
!macro NSIS_HOOK_PREINSTALL
  IfFileExists "%ROOT%\request-late-lock" 0 atlas_lock_done
  ; Synchronize with the runner only after the old host handed off. Make this
  ; expected installer refusal silent so the job can resume the previous build.
  SetSilent silent
  System::Call 'kernel32::GetCurrentProcessId() i.r0'
  FileOpen $1 "%ROOT%\installer-started" w
  FileWrite $1 "$0"
  FileClose $1
  StrCpy $0 300
  atlas_wait_lock:
    IfFileExists "%ROOT%\lock-ready" atlas_lock_done
    Sleep 100
    IntOp $0 $0 - 1
    IntCmp $0 0 atlas_lock_timeout atlas_lock_timeout atlas_wait_lock
  atlas_lock_timeout:
    SetErrorLevel 2
    Quit
  atlas_lock_done:
!macroend
!macro NSIS_HOOK_POSTINSTALL
  IfFileExists "%ROOT%\qualify-lifecycle" atlas_skip_updater_hook
  ClearErrors
  ExecWait "$\"%PWSH%$\" -NoProfile -File $\"%HOOK%$\" -QualificationRoot $\"%ROOT%$\" -InstallDirectory $\"$INSTDIR$\"" $0
  IfErrors atlas_hook_failed
  StrCmp $0 0 0 atlas_hook_failed
  Exec "$\"%PWSH%$\" -NoProfile -File $\"%HOOK%$\" -QualificationRoot $\"%ROOT%$\" -InstallDirectory $\"$INSTDIR$\" -Restart"
  IfErrors atlas_hook_failed
  SetErrorLevel 0
  Quit
  atlas_hook_failed:
    SetErrorLevel 1
    Abort "Atlas qualification POSTINSTALL hook failed"
  atlas_skip_updater_hook:
!macroend
'@
    $installerHook = (Get-Item -LiteralPath (Join-Path $PSScriptRoot 'src-tauri\windows-installer.nsh')).FullName
    $hookScript = Join-Path $PSScriptRoot 'qualify-windows.hook.ps1'
    $paths = @{
        '%ATLAS_INSTALLER_HOOK%' = $installerHook
        '%DIRECT%' = Get-NsisRuntimePath $directInstaller
        '%PWSH%' = Get-NsisRuntimePath (Get-Command pwsh -CommandType Application).Source
        '%HOOK%' = Get-NsisRuntimePath $hookScript
        '%ROOT%' = Get-NsisRuntimePath $root
    }
    $macro = [regex]::Replace($macro, '%(ATLAS_INSTALLER_HOOK|DIRECT|PWSH|HOOK|ROOT)%', { param($match) $paths[$match.Value] })
    $macro | Set-Content -LiteralPath $hooks -Encoding utf8BOM
    $template = Join-Path $root 'qualify-installer.nsi'
    (Get-Content -LiteralPath "$PSScriptRoot/src-tauri/installer.nsi" -Raw).Replace('!insertmacro AtlasPayload ', '!insertmacro QualifyAtlasPayload ') |
        Set-Content -LiteralPath $template -Encoding utf8
    $config = Join-Path $root 'target-config.json'
    @{ version = $nightly; bundle = @{ windows = @{ nsis = @{ installerHooks = $hooks; template = $template } } } } |
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
    $lifecycle = Join-Path $root 'qualify-lifecycle'
    [IO.File]::WriteAllText($lifecycle, '')
    try {
        & "$PSScriptRoot/verify-windows-installer.ps1" -Installer $targetInstaller -ExpectedCli $cli -ScratchDirectory $root
        foreach ($evidence in 'setup-refused-uninstall', 'uninstall-refused-setup') {
            if ([IO.File]::ReadAllText((Join-Path $root $evidence)) -ne '1618') { throw "Missing lifecycle evidence: $evidence" }
        }
    } finally { [IO.File]::Delete($lifecycle) }
    Invoke-Checked node @('desktop/updater-package.mjs', 'sign', "v$nightly", $targetInstaller)
    $targetHostHash = Get-NsisHostHash "$PSScriptRoot/src-tauri/target/x86_64-pc-windows-msvc/release/atlas-desktop.exe"
    @{
        root = $root; oldInstaller = $oldInstaller; installer = $targetInstaller; stable = $stable; nightly = $nightly
        commit = $commit; port = $port; tlsKey = $tlsKey; tlsCert = $tlsCert
        priorInstaller = $priorInstaller; priorTag = $priorTag; priorHash = $priorHash
        directInstaller = $directInstaller; directHostHash = $directHostHash
        oldHostSha256 = $oldHostHash; oldCliSha256 = (Get-FileHash $oldCli).Hash.ToLowerInvariant()
        targetHostSha256 = $targetHostHash; targetCliSha256 = (Get-FileHash $cli).Hash.ToLowerInvariant()
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $root 'qualification.json') -Encoding utf8
    Invoke-Checked node @('desktop/qualify-windows.mjs', (Join-Path $root 'qualification.json'))
}
finally {
    Pop-Location
    foreach ($name in $original.Keys) { [Environment]::SetEnvironmentVariable($name, $original[$name]) }
}
