param([switch]$PathOnly)

$ErrorActionPreference = 'Stop'
if ($env:ATLAS_CLEAN_WINDOWS_CONTAINER -ne '1' -or [Environment]::UserName -ne 'ContainerUser') {
    throw 'Run this rehearsal as ContainerUser in a fresh Windows container. It changes the disposable user PATH.'
}
Write-Output "Qualifying Windows $([Environment]::OSVersion.Version) as $([Environment]::UserName) with PowerShell $($PSVersionTable.PSVersion)"
$assets = $PSScriptRoot
$env:ATLAS_QUALIFICATION_ASSETS = $assets
$manifest = Get-Content -Raw -LiteralPath "$assets\atlas-release-manifest.json" | ConvertFrom-Json
$version = $manifest.version.Substring(1)
$tag = if ($version -match '-nightly\.(\d{14}\.g[0-9a-f]{12})$') { 'nightly/' + $Matches[1] } else { $manifest.version }
$root = Join-Path $env:TEMP ('atlas-clean-' + [Guid]::NewGuid())
New-Item -Path $root -ItemType Directory | Out-Null
$index = Join-Path $root 'workspace.sqlite'
Set-Location -LiteralPath $root

# Reject a contaminated baseline before launching any shipped executable.
if ($PathOnly) {
    # PowerShell 7 adds its own directory to PATH. Keep the mounted host's
    # private dependencies out of every Atlas child process's loader search.
    $env:Path = (($env:Path -split ';') | Where-Object { $_ -and $_ -ne $PSHOME }) -join ';'
}
if ([Runtime.InteropServices.RuntimeInformation]::OSArchitecture -ne 'X64') { throw 'Expected Windows x64' }
foreach ($dll in 'dxgi.dll', 'd3d12.dll', 'DirectML.dll') {
    if (!(Test-Path -LiteralPath (Join-Path ([Environment]::SystemDirectory) $dll) -PathType Leaf)) {
        throw "Clean Windows image is missing the desktop system component $dll required by the shipped CLI."
    }
}
if (Get-Command cl.exe, msbuild.exe, cargo.exe -ErrorAction SilentlyContinue) { throw 'Build tools found on PATH' }
if (Test-Path -LiteralPath "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe") { throw 'Visual Studio installer found' }
foreach ($view in 'HKLM:\SOFTWARE', 'HKLM:\SOFTWARE\Wow6432Node') {
    $redist = Get-ItemProperty -LiteralPath (Join-Path $view 'Microsoft\VisualStudio\14.0\VC\Runtimes\x64') -ErrorAction SilentlyContinue
    if ($redist.Installed -eq 1) { throw 'Machine-wide x64 Visual C++ runtime is installed' }
}
foreach ($dir in @("$env:SystemRoot\System32") + @($env:Path -split ';' | Where-Object { $_ })) {
    foreach ($dll in 'msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll') {
        if (Test-Path -LiteralPath (Join-Path $dir.Trim('"') $dll)) { throw "Unexpected preinstalled runtime: $dir\$dll" }
    }
}

function Test-Cli([string]$Binary, [string]$Variant) {
    $reported = (& $Binary --version) -join [Environment]::NewLine
    $exitCode = $LASTEXITCODE
    if ($reported -cne "atlas $version" -or $exitCode -ne 0) {
        $status = '{0} (0x{0:X8})' -f $exitCode
        throw "$Binary --version exited $status. Expected 'atlas $version', received '$reported'."
    }
    $info = (& $Binary build-info --json) | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or $info.status -ne 'ok' -or $info.data.variant -cne $Variant -or $info.data.version -cne $version) {
        throw "Unexpected $Variant build metadata"
    }
    $help = (& $Binary --help) -join [Environment]::NewLine
    if ($LASTEXITCODE -ne 0 -or ($help -match '(?m)^\s+web\s') -ne ($Variant -eq 'full')) { throw 'Unexpected web command surface' }
    if ($PathOnly) { return }
    if (!(Test-Path -LiteralPath $index)) {
        $setup = (& $Binary setup --offline --no-embeddings --source "$assets\source" --index $index --path-mode global --json) | ConvertFrom-Json
        if ($LASTEXITCODE -ne 0 -or !$setup.data.ready) { throw "Could not prepare the offline source fixture: $($setup | ConvertTo-Json -Depth 5)" }
    }
    $search = (& $Binary search Demoralize --retrieval fts --path-mode global --index $index --json) | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0 -or $search.data.rows.record.record_key -notcontains 'actions:0000000000000001') { throw 'Fixture search failed' }
    if ($Variant -ne 'full') { return }

    $log = Join-Path $root 'web.log'
    $errorLog = Join-Path $root 'web-error.log'
    $process = Start-Process -FilePath $Binary -PassThru -WindowStyle Hidden -RedirectStandardOutput $log -RedirectStandardError $errorLog -ArgumentList @(
        'web', '--port', '0', '--path-mode', 'global', '--index', ('"' + $index + '"'), '--source', "$assets\source"
    )
    try {
        $timeout = [Diagnostics.Stopwatch]::StartNew()
        while ($timeout.Elapsed.TotalSeconds -lt 30 -and !$process.HasExited) {
            $output = Get-Content -Raw -LiteralPath $log
            if ($output -match 'Atlas web service listening at (http://127\.0\.0\.1:\d+)') {
                $page = Microsoft.PowerShell.Utility\Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 -Uri $Matches[1]
                if ($page.StatusCode -ne 200 -or $page.Content -notmatch '/assets/[^"<> ]+\.js') { throw 'Invalid embedded web page' }
                return
            }
            Start-Sleep -Milliseconds 150
        }
        throw "Web startup failed: $(Get-Content -Raw -LiteralPath $errorLog)"
    } finally {
        if (!$process.HasExited) { $process.Kill(); $process.WaitForExit() }
        $process.Dispose()
    }
}

# Serve the selected immutable release from the read-only artifact mount. All
# checksum, manifest, version and file replacement logic runs in the published PS1.
function Invoke-WebRequest {
    param([Uri]$Uri, [string]$OutFile, [switch]$UseBasicParsing)
    $name = [IO.Path]::GetFileName($Uri.AbsolutePath)
    Copy-Item -LiteralPath (Join-Path $env:ATLAS_QUALIFICATION_ASSETS $name) -Destination $OutFile
}
$install = Join-Path $root ("O'Brien " + [char]0x2019 + ' $(bin) & ' + [char]0x201C + 'Atlas' + [char]0x201D)
foreach ($variant in 'full', 'cli-only') {
    $asset = @($manifest.assets | Where-Object { $_.target -eq 'x86_64-pc-windows-msvc' -and $_.variant -eq $variant })
    if ($asset.Count -ne 1) { throw "Missing Windows $variant ZIP" }
    if (!$PathOnly) {
        $zip = Join-Path $assets $asset[0].name
        $digest = (Get-FileHash -LiteralPath $zip).Hash.ToLowerInvariant()
        $sums = Get-Content -LiteralPath (Join-Path $assets 'SHA256SUMS')
        if ($digest -ne $asset[0].sha256 -or $sums -cnotcontains "$digest  $($asset[0].name)") { throw "Manual ZIP checksum mismatch: $zip" }
        $manual = Join-Path $root "manual-$variant"
        Expand-Archive -LiteralPath $zip -DestinationPath $manual
        $binary = @(Get-ChildItem -LiteralPath $manual -File -Recurse -Filter atlas.exe)
        if ($binary.Count -ne 1) { throw "Expected one manual $variant executable" }
        Test-Cli $binary[0].FullName $variant
        Write-Output "Manual $variant $version passed."
    }
    $options = @{}
    if ($variant -eq 'cli-only') { $options.WithoutWeb = $true }
    $output = (& "$assets\atlas-installer.ps1" -Version $tag -InstallDir $install -Yes @options 6>&1 | Out-String -Width 4096) -split '\r?\n'
    Test-Cli (Join-Path $install 'atlas.exe') $variant
    Write-Output "Scripted $variant $version passed."
    if ($PathOnly) { break }
}

# Keep other Atlas DLLs off PATH throughout the clean-loader checks above.
$code = @()
$collect = $false
foreach ($line in $output) {
    if ($line -eq 'PowerShell hint:') { $collect = $true; continue }
    if ($collect -and !$line.StartsWith('  ')) { break }
    if ($collect) { $code += $line }
}
if (!$code.Count) { throw 'No copyable PATH hint in installer output' }
$key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
try {
    $raw = '%USERPROFILE%\existing bin;C:\another user bin'
    $key.SetValue('Path', $raw, 'ExpandString')
    $apply = [ScriptBlock]::Create(($code -join [Environment]::NewLine))
    & $apply
    & $apply
    if ($key.GetValueKind('Path') -ne 'ExpandString' -or $key.GetValue('Path', '', 'DoNotExpandEnvironmentNames') -cne "$raw;$install" -or
        (Get-Command atlas.exe).Source -cne (Join-Path $install 'atlas.exe')) { throw 'User PATH or command resolution changed unexpectedly' }
} finally { $key.Dispose() }
Write-Output "Copied PATH hint passed on PowerShell $($PSVersionTable.PSVersion)."
