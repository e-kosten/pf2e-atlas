param([Parameter(Mandatory)][string]$Binary)

$ErrorActionPreference = 'Stop'
$binaryPath = (Get-Item -LiteralPath $Binary).FullName
$directory = Split-Path -Parent $binaryPath
$receipt = Get-Content -LiteralPath (Join-Path $directory 'atlas-windows-runtime.json') -Raw | ConvertFrom-Json
if ($receipt.format -cne 'atlas-windows-runtime-v1') { throw 'Unexpected Windows runtime receipt' }

# Check the imports of the shipped executable and DLLs. A developer machine
# with a system-wide redistributable could otherwise conceal a missing DLL.
if ($env:VCToolsInstallDir) {
    $linker = Join-Path $env:VCToolsInstallDir 'bin/Hostx64/x64/link.exe'
} else {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $linker = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find 'VC\Tools\MSVC\*\bin\Hostx64\x64\link.exe' |
        Select-Object -First 1
}
if (!$linker) { throw 'Could not find the x64 Visual C++ linker to inspect packaged imports' }
$payload = @($binaryPath)
foreach ($name in 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
    $path = Join-Path $directory $name
    if ((Get-FileHash -LiteralPath $path).Hash -ne $receipt.files.$name) { throw "Invalid packaged runtime: $name" }
    $payload += $path
}
foreach ($path in $payload) {
    $imports = & $linker /dump /dependents $path
    if ($LASTEXITCODE -ne 0) { throw "Cannot inspect packaged imports of $path" }
    foreach ($line in $imports) {
        if ($line.Trim() -match '^(?:msvcp|msvcr|vcruntime|concrt|vcomp|vcamp|vccorlib)\d+[\w.-]*\.dll$') {
            if (!(Test-Path -LiteralPath (Join-Path $directory $Matches[0]) -PathType Leaf)) {
                throw "Missing app-local import $($Matches[0]) required by $path"
            }
        }
    }
}
