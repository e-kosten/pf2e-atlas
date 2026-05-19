param(
  [string]$InstallDir = $env:ATLAS_INSTALL_DIR,
  [string]$Version = $env:ATLAS_VERSION,
  [switch]$Yes,
  [switch]$DryRun
)

$ErrorActionPreference = "Stop"

if (-not $Version) { $Version = "latest" }
if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\PF2e Atlas\bin" }
$Repo = if ($env:ATLAS_REPO) { $env:ATLAS_REPO } else { "e-kosten/pf2e-atlas" }
if ($env:ATLAS_YES -eq "1") { $Yes = $true }

function Fail($Message) {
  Write-Error $Message
  Write-Host ""
  Write-Host "Manual install fallback:"
  Write-Host "  1. Open https://github.com/$Repo/releases"
  Write-Host "  2. Download the Windows archive and SHA256SUMS"
  Write-Host "  3. Verify the checksum, extract atlas.exe, and place it on PATH"
  Write-Host "  4. Run atlas setup"
  Write-Host ""
  Write-Host "Source fallback for contributors with Rust 1.95+:"
  Write-Host "  git clone https://github.com/$Repo.git"
  Write-Host "  cd pf2e-atlas"
  Write-Host "  cargo install --path crates/atlas-cli --locked"
  exit 1
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
switch ($arch) {
  "X64" { $target = "x86_64-pc-windows-msvc" }
  "Arm64" { $target = "aarch64-pc-windows-msvc" }
  default { Fail "unsupported Windows architecture: $arch" }
}

if ($Version -eq "latest") {
  $baseUrl = "https://github.com/$Repo/releases/latest/download"
} elseif ($Version.StartsWith("v")) {
  $baseUrl = "https://github.com/$Repo/releases/download/$Version"
} else {
  $baseUrl = "https://github.com/$Repo/releases/download/v$Version"
}

$manifestUrl = "$baseUrl/atlas-release-manifest.json"
$archive = "(from manifest)"
$installPath = Join-Path $InstallDir "atlas.exe"
$pathEntries = ($env:PATH -split ';') | Where-Object { $_ -ne "" }
$pathStatus = if ($pathEntries -contains $InstallDir) { "on-path" } else { "not-on-path" }
$existing = if (Test-Path $installPath) { "at-destination" } else { "none" }
$existingVersion = "unknown"
if (Test-Path $installPath) {
  try { $existingVersion = & $installPath --version } catch { $existingVersion = "unknown" }
}

$shadow = $null
$cmd = Get-Command atlas.exe -ErrorAction SilentlyContinue
if ($cmd -and ($cmd.Source -ne $installPath)) { $shadow = $cmd.Source }

Write-Host "Atlas installer"
Write-Host "  target:      $target"
Write-Host "  version:     $Version"
Write-Host "  manifest:    $manifestUrl"
Write-Host "  install dir: $InstallDir"
Write-Host "  PATH:        $pathStatus"
Write-Host "  existing:    $existing"
if ($existingVersion -ne "unknown") { Write-Host "  current:     $existingVersion" }
if ($shadow) { Write-Host "  warning: another atlas appears on PATH at $shadow" }

if ($DryRun) {
  Write-Host "Dry run only; no download or install performed."
  exit 0
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("atlas-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null
try {
  $sums = Join-Path $tmp "SHA256SUMS"
  $manifestPath = Join-Path $tmp "atlas-release-manifest.json"
  $zip = Join-Path $tmp $archive
  Invoke-WebRequest -Uri "$baseUrl/SHA256SUMS" -OutFile $sums
  Invoke-WebRequest -Uri $manifestUrl -OutFile $manifestPath
  $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
  $asset = $manifest.assets | Where-Object { $_.target -eq $target } | Select-Object -First 1
  if (-not $asset) { Fail "manifest did not contain an asset for $target" }
  $archive = $asset.name
  $zip = Join-Path $tmp $archive
  Write-Host "  asset:       $archive"
  Invoke-WebRequest -Uri "$baseUrl/$archive" -OutFile $zip

  $line = Get-Content $sums | Where-Object {
    $parts = $_ -split '\s+', 2
    $parts.Count -eq 2 -and $parts[1] -eq $archive
  } | Select-Object -First 1
  if (-not $line) { Fail "checksum for $archive not found in SHA256SUMS" }
  $expected = ($line -split '\s+')[0].ToLowerInvariant()
  $actual = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLowerInvariant()
  if ($expected -ne $actual) { Fail "checksum mismatch for $archive" }

  $extract = Join-Path $tmp "extract"
  Expand-Archive -Path $zip -DestinationPath $extract
  $newBinary = Get-ChildItem -Path $extract -Filter atlas.exe -Recurse | Select-Object -First 1
  if (-not $newBinary) { Fail "archive did not contain atlas.exe" }
  $newVersion = & $newBinary.FullName --version
  Write-Host "  target bin:  $newVersion"

  function VersionNumber($Text) {
    if (-not $Text -or $Text -eq "unknown") { return "unknown" }
    $parts = $Text -split '\s+'
    return $parts[$parts.Length - 1]
  }

  function ClassifyChange($OldText, $NewText) {
    $old = VersionNumber $OldText
    $new = VersionNumber $NewText
    if ($old -eq "unknown" -or $new -eq "unknown") { return "unknown" }
    if ($old -eq $new) { return "same-version" }
    try {
      $oldSemver = [System.Management.Automation.SemanticVersion]::Parse($old)
      $newSemver = [System.Management.Automation.SemanticVersion]::Parse($new)
      if ($newSemver -gt $oldSemver) { return "upgrade" }
      if ($newSemver -lt $oldSemver) { return "downgrade" }
      return "same-version"
    } catch {
      return "unknown"
    }
  }

  if (Test-Path $installPath) {
    $change = ClassifyChange $existingVersion $newVersion
    Write-Host "  action:      replace existing atlas ($change)"
    if (-not $Yes) {
      $answer = Read-Host "Replace existing atlas at $installPath? [y/N]"
      if ($answer -notin @("y", "Y", "yes", "YES")) { Fail "install cancelled" }
    }
  }

  $tmpDest = Join-Path $InstallDir ".atlas.install.$PID.exe"
  Copy-Item $newBinary.FullName $tmpDest -Force
  $backup = $null
  if (Test-Path $installPath) {
    $backup = Join-Path $InstallDir ".atlas.backup.$PID.exe"
    Copy-Item $installPath $backup -Force
  }
  try {
    Move-Item $tmpDest $installPath -Force
    & $installPath --version | Out-Null
  } catch {
    if ($backup -and (Test-Path $backup)) { Move-Item $backup $installPath -Force }
    Fail "failed to replace or validate atlas.exe; close running atlas processes and rerun"
  }
  if ($backup -and (Test-Path $backup)) { Remove-Item $backup -Force }
} finally {
  Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "Installed atlas to $installPath"
if ($pathStatus -ne "on-path") {
  Write-Host ""
  Write-Host "$InstallDir is not on PATH. Add it to your user PATH, then open a new terminal."
  Write-Host "PowerShell hint:"
  Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$InstallDir', 'User')"
}
