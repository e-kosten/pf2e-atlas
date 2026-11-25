[CmdletBinding(DefaultParameterSetName = 'Local')]
param(
    [Parameter(Mandatory, ParameterSetName = 'Release')]
    [string]$PackagedCli,
    [Parameter(Mandatory, ParameterSetName = 'Release')]
    [string]$Version,
    [Parameter(Mandatory, ParameterSetName = 'Release')]
    [string]$OutputInstaller,

    [string]$HostTargetDirectory,
    [switch]$CargoTimings
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$target = 'x86_64-pc-windows-msvc'
if ($env:OS -ne 'Windows_NT' -or
    [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') {
    throw 'The Atlas Windows installer requires a Windows x64 build machine.'
}
$release = $PSCmdlet.ParameterSetName -eq 'Release'
if ($release) {
    $PackagedCli = (Resolve-Path -LiteralPath $PackagedCli).Path
    $OutputInstaller = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutputInstaller)
}

function Invoke-BuildStep {
    param([string]$Command, [string[]]$Arguments)
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Command failed with exit code $LASTEXITCODE."
    }
}

$previousDesktopRelease = $env:ATLAS_DESKTOP_RELEASE
$previousDesktopSourceCommit = $env:ATLAS_DESKTOP_SOURCE_COMMIT
$previousWorkspaceSchema = $env:ATLAS_WORKSPACE_SCHEMA
$previousTargetDir = $env:CARGO_TARGET_DIR
$previousRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
Push-Location $root
try {
    $env:CARGO_TARGET_DIR = Join-Path $root 'target'
    $runtime = Join-Path $PSScriptRoot '.generated/windows-crt'
    & "$root/scripts/release/stage-windows-crt.ps1" -Destination $runtime | Out-Null
    if (!$release) {
        Invoke-BuildStep npm.cmd @('--prefix', 'web/atlas-ui', 'run', 'build')
    }
    Invoke-BuildStep node @('desktop/prepare-bootstrap.mjs')
    Invoke-BuildStep npm.cmd @('--prefix', 'dev-tools/release', 'run', 'notices', '--',
        '--output', (Join-Path $PSScriptRoot '.generated/THIRD-PARTY-NOTICES.md'), '--desktop')
    $cargoOptions = @('--locked')
    if ($CargoTimings) { $cargoOptions += '--timings' }
    if (!$release) {
        Invoke-BuildStep cargo (@('build', '--release', '--target', $target, '-p', 'atlas-cli') + $cargoOptions)
        $PackagedCli = Join-Path $root "target/$target/release/atlas.exe"
    }

    $binaries = Join-Path $PSScriptRoot 'src-tauri/binaries'
    New-Item -ItemType Directory -Force $binaries | Out-Null
    Copy-Item -LiteralPath $PackagedCli -Destination "$binaries/atlas-$target.exe"

    # Apply the static CRT required by the WebView2 loader instead of the
    # workspace flags for the CLI's ONNX Runtime.
    $env:CARGO_ENCODED_RUSTFLAGS = '-Ctarget-feature=+crt-static'
    if ($HostTargetDirectory) {
        $env:CARGO_TARGET_DIR = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($HostTargetDirectory)
    }
    Set-Location $PSScriptRoot
    $arguments = @('build', '--target', $target, '--bundles', 'nsis', '--no-sign')
    if ($release) {
        $config = Join-Path $PSScriptRoot '.generated/windows-release.json'
        @{ version = $Version } | ConvertTo-Json -Compress | Set-Content -LiteralPath $config -Encoding ascii
        $arguments += @('--config', $config)
        if ($env:ATLAS_UPDATER_PUBLIC_KEY) {
            $env:ATLAS_DESKTOP_RELEASE = $Version
            $env:ATLAS_DESKTOP_SOURCE_COMMIT = git rev-parse HEAD
            if ($LASTEXITCODE -ne 0) { throw 'Could not identify the desktop build source.' }
            $env:ATLAS_WORKSPACE_SCHEMA = node ./updater-package.mjs compatibility "v$Version" $PackagedCli "$OutputInstaller.compatibility.json"
            if ($LASTEXITCODE -ne 0) { throw 'Could not read the bundled Atlas update compatibility.' }
        }
    }
    Invoke-BuildStep './node_modules/.bin/tauri.cmd' ($arguments + @('--') + $cargoOptions)
    if ($release) {
        $installer = Join-Path $env:CARGO_TARGET_DIR "$target/release/bundle/nsis/PF2e Atlas_${Version}_x64-setup.exe"
        Copy-Item -LiteralPath $installer -Destination $OutputInstaller
    }
}
finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previousRustFlags
    $env:CARGO_TARGET_DIR = $previousTargetDir
    $env:ATLAS_DESKTOP_RELEASE = $previousDesktopRelease
    $env:ATLAS_DESKTOP_SOURCE_COMMIT = $previousDesktopSourceCommit
    $env:ATLAS_WORKSPACE_SCHEMA = $previousWorkspaceSchema
    Pop-Location
}
