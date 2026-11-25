[CmdletBinding()]
param([Parameter(Mandatory)][string]$Destination, [switch]$IncludeReceipt)

$ErrorActionPreference = 'Stop'
if ($env:VCToolsRedistDir) {
    $redist = $env:VCToolsRedistDir
} else {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($installation) {
        $redist = Get-ChildItem (Join-Path $installation 'VC/Redist/MSVC') -Directory |
            Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName 'x64/Microsoft.VC143.CRT') } |
            Sort-Object { [version]$_.Name } -Descending |
            Select-Object -First 1 -ExpandProperty FullName
    }
}
if (!$redist) { throw 'Set VCToolsRedistDir to the Visual C++ v143 x64 redistributable version folder.' }
$crt = Join-Path $redist 'x64/Microsoft.VC143.CRT'
New-Item -ItemType Directory -Force -Path $Destination | Out-Null
$hashes = [ordered]@{}
foreach ($name in 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
    Copy-Item -LiteralPath (Join-Path $crt $name) -Destination $Destination
    $hashes[$name] = (Get-FileHash -LiteralPath (Join-Path $Destination $name)).Hash
    $name
}
if ($IncludeReceipt) {
    $receipt = 'atlas-windows-runtime.json'
    @{ format = 'atlas-windows-runtime-v1'; files = $hashes } | ConvertTo-Json |
        Set-Content -LiteralPath (Join-Path $Destination $receipt) -Encoding ascii
    $receipt
}
