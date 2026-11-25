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

foreach ($file in 'atlas-desktop-windows-runtime.json', 'atlas-desktop.exe', 'atlas.exe', 'THIRD-PARTY-NOTICES.md', 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
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

# The registered installation cannot authorize replacements in another folder.
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\PF2e Atlas'
$registration = (Get-ItemProperty -LiteralPath $uninstallKey).InstallLocation
foreach ($name in 'vcruntime140.dll', 'uninstall.exe') {
    $other = Join-Path $scratch ('Atlas collision ' + [guid]::NewGuid().ToString('N'))
    [void][IO.Directory]::CreateDirectory($other)
    $existing = Join-Path $other $name
    [IO.File]::Copy((Join-Path $installDir $name), $existing)
    $before = (Get-FileHash -LiteralPath $existing).Hash
    $refusal = Start-Process $Installer -ArgumentList "/S /D=$other" -PassThru -Wait
    if ($refusal.ExitCode -eq 0 -or (Get-FileHash -LiteralPath $existing).Hash -ne $before -or
        @(Get-ChildItem -Force -LiteralPath $other).Count -ne 1 -or
        (Get-ItemProperty -LiteralPath $uninstallKey).InstallLocation -ne $registration) {
        throw "Installer did not preserve $existing and its previous registration"
    }
    [IO.File]::Delete($existing)
    [IO.Directory]::Delete($other)
}
& $cli --version
if ($LASTEXITCODE) { throw 'Refusal damaged the existing CLI' }
$unrelated = Join-Path $installDir 'local-file.txt'
[IO.File]::WriteAllText($unrelated, 'retain')
$changed = Join-Path $installDir 'vcruntime140.dll'
[IO.File]::WriteAllText($changed, 'replaced after installation')
$uninstaller = Join-Path $installDir 'uninstall.exe'
# _?= waits on the uninstaller itself instead of its temporary launcher.
$removed = Start-Process $uninstaller -ArgumentList "/S _?=$installDir" -PassThru -Wait
if ($removed.ExitCode -ne 0 -or [IO.File]::ReadAllText($changed) -ne 'replaced after installation' -or
    [IO.File]::ReadAllText($unrelated) -ne 'retain' -or (Test-Path -LiteralPath $uninstallKey)) {
    throw 'Uninstaller failed to preserve unrelated files or remove its registration'
}
foreach ($name in 'atlas.exe','atlas-desktop.exe','msvcp140.dll','msvcp140_1.dll','vcruntime140_1.dll','atlas-desktop-windows-runtime.json') {
    if (Test-Path -LiteralPath (Join-Path $installDir $name)) { throw "Uninstaller left its owned file $name" }
}
