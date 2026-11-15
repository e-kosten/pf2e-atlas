[CmdletBinding(DefaultParameterSetName = 'Local')]
param(
    [Parameter(Mandatory, ParameterSetName = 'Release')]
    [string]$PackagedCli,
    [Parameter(Mandatory, ParameterSetName = 'Release')]
    [string]$Version,
    [Parameter(Mandatory, ParameterSetName = 'Release')]
    [string]$OutputInstaller
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

function Get-DesktopCrt {
    if ($env:VCToolsRedistDir) {
        return (Join-Path $env:VCToolsRedistDir 'x64/Microsoft.VC143.CRT')
    }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (Test-Path -LiteralPath $vswhere) {
        $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($installation) {
            $redist = Get-ChildItem (Join-Path $installation 'VC/Redist/MSVC') -Directory |
                Where-Object { Test-Path (Join-Path $_.FullName 'x64/Microsoft.VC143.CRT') } |
                Sort-Object { [version]$_.Name } -Descending |
                Select-Object -First 1
            if ($redist) {
                return (Join-Path $redist.FullName 'x64/Microsoft.VC143.CRT')
            }
        }
    }
    throw 'Locate the Visual C++ v143 x64 redistributable and set VCToolsRedistDir to its version folder.'
}

$previousTargetDir = $env:CARGO_TARGET_DIR
$previousRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
Push-Location $root
try {
    $env:CARGO_TARGET_DIR = Join-Path $root 'target'
    $crt = Get-DesktopCrt
    $runtime = Join-Path $PSScriptRoot '.generated/windows-crt'
    New-Item -ItemType Directory -Force $runtime | Out-Null
    foreach ($file in 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
        Copy-Item -LiteralPath (Join-Path $crt $file) -Destination $runtime
    }
    if (!$release) {
        Invoke-BuildStep npm.cmd @('--prefix', 'web/atlas-ui', 'run', 'build')
    }
    Invoke-BuildStep node @('desktop/prepare-bootstrap.mjs')
    Invoke-BuildStep npm.cmd @('--prefix', 'dev-tools/release', 'run', 'notices', '--',
        '--output', (Join-Path $PSScriptRoot '.generated/THIRD-PARTY-NOTICES.md'), '--desktop')
    if (!$release) {
        Invoke-BuildStep cargo @('build', '--locked', '--release', '--target', $target, '-p', 'atlas-cli')
        $PackagedCli = Join-Path $root "target/$target/release/atlas.exe"
    }

    $binaries = Join-Path $PSScriptRoot 'src-tauri/binaries'
    New-Item -ItemType Directory -Force $binaries | Out-Null
    Copy-Item -LiteralPath $PackagedCli -Destination "$binaries/atlas-$target.exe"

    # Apply the static CRT required by the WebView2 loader instead of the
    # workspace flags for the CLI's ONNX Runtime.
    $env:CARGO_ENCODED_RUSTFLAGS = '-Ctarget-feature=+crt-static'
    Set-Location $PSScriptRoot
    $arguments = @('build', '--target', $target, '--bundles', 'nsis', '--no-sign')
    if ($release) {
        $config = Join-Path $PSScriptRoot '.generated/windows-release.json'
        @{ version = $Version } | ConvertTo-Json -Compress | Set-Content -LiteralPath $config -Encoding ascii
        $arguments += @('--config', $config)
    }
    Invoke-BuildStep './node_modules/.bin/tauri.cmd' ($arguments + @('--', '--locked'))
    if ($release) {
        $installer = Join-Path $env:CARGO_TARGET_DIR "$target/release/bundle/nsis/PF2e Atlas_${Version}_x64-setup.exe"
        Copy-Item -LiteralPath $installer -Destination $OutputInstaller
    }
}
finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previousRustFlags
    $env:CARGO_TARGET_DIR = $previousTargetDir
    Pop-Location
}
