[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [string]$Installer,
    [Parameter(Mandatory)]
    [string]$ExpectedCli,
    [Parameter(Mandatory)]
    [string]$ScratchDirectory
)

$ErrorActionPreference = 'Stop'
$Installer = (Resolve-Path -LiteralPath $Installer).Path
$scratch = (Resolve-Path -LiteralPath $ScratchDirectory).Path
$installDir = Join-Path $scratch ('Atlas Desktop Test ' + [guid]::NewGuid().ToString('N'))
$process = Start-Process $Installer -ArgumentList "/S /D=$installDir" -WindowStyle Hidden -PassThru -Wait
if ($process.ExitCode -ne 0) { throw "Setup failed with code $($process.ExitCode)." }

foreach ($file in 'atlas-desktop.exe', 'atlas.exe', 'THIRD-PARTY-NOTICES.md', 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
    if (!(Test-Path -LiteralPath (Join-Path $installDir $file) -PathType Leaf)) {
        throw "Missing installed file $file."
    }
}
$registered = (Get-Item 'HKCU:\Software\Classes\pf2e-atlas\shell\open\command').GetValue('')
if ($registered -ne ('"{0}\atlas-desktop.exe" "%1"' -f $installDir)) {
    throw "Unexpected protocol command $registered."
}
$cli = Join-Path $installDir 'atlas.exe'
if ((Get-FileHash -LiteralPath $ExpectedCli).Hash -ne (Get-FileHash -LiteralPath $cli).Hash) {
    throw 'Installed sidecar differs from the matching CLI.'
}
$previousLocalAppData = $env:LOCALAPPDATA
try {
    $env:LOCALAPPDATA = Join-Path $installDir 'probe-workspace'
    $probe = & $cli desktop-probe | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or $probe.data.version -ne 1 -or $probe.data.status -ne 'needs_setup') {
        throw 'The bundled command did not recognize the empty workspace.'
    }
}
finally {
    $env:LOCALAPPDATA = $previousLocalAppData
}
