param(
  [string]$InstallDir = $env:ATLAS_INSTALL_DIR,
  [string]$Version = $env:ATLAS_VERSION,
  [switch]$WithoutWeb,
  [switch]$WithWeb,
  [switch]$Yes,
  [switch]$DryRun
)

$ErrorActionPreference = "Stop"

if (-not $Version) { $Version = "latest" }
if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA "Programs\PF2e Atlas\bin" }
$InstallDir = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($InstallDir)
$Repo = if ($env:ATLAS_REPO) { $env:ATLAS_REPO } else { "e-kosten/pf2e-atlas" }
if ($env:ATLAS_YES -eq "1") { $Yes = $true }

function Fail($Message) {
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
  if ($WithoutWeb) {
    Write-Host "  cargo install --path crates/atlas-cli --locked --no-default-features"
  } else {
    Write-Host "  npm --prefix web/atlas-ui ci"
    Write-Host "  npm --prefix web/atlas-ui run build"
    Write-Host "  cargo install --path crates/atlas-cli --locked"
  }
  Write-Error $Message -ErrorAction Stop
}

if ($WithoutWeb -and $WithWeb) { Fail "-WithWeb and -WithoutWeb cannot be combined" }

function Get-BuildVariant($Binary) {
  # Older releases do not provide build-info and are full builds.
  $previousNativePreference = $PSNativeCommandUseErrorActionPreference
  $PSNativeCommandUseErrorActionPreference = $false
  # Windows PowerShell can treat redirected native stderr as a PowerShell error.
  $ErrorActionPreference = 'Continue'
  try {
    $metadata = & $Binary build-info --json 2>$null
    if ($LASTEXITCODE -ne 0) { return "legacy" }
    $info = ($metadata -join "`n") | ConvertFrom-Json
    if ($info.status -ne "ok" -or $info.data.variant -cnotin @("full", "cli-only")) {
      throw "invalid build information"
    }
    return $info.data.variant
  } finally {
    $PSNativeCommandUseErrorActionPreference = $previousNativePreference
  }
}

$arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
switch ($arch) {
  "X64" { $target = "x86_64-pc-windows-msvc" }
  "Arm64" { Fail "unsupported Windows architecture: Arm64; Atlas release binaries currently require Windows x64" }
  default { Fail "unsupported Windows architecture: $arch" }
}

$tag = switch -CaseSensitive -Regex ($Version) {
  '^nightly$' { 'nightly/latest'; break }
  '^(latest$|(?:release|rc|nightly)/)' { $Version; break }
  '-rc\.' { 'rc/v' + ($Version -creplace '^v', ''); break }
  default { 'release/v' + ($Version -creplace '^v', '') }
}
if ($tag -ceq "latest") {
  $baseUrl = "https://github.com/$Repo/releases/latest/download"
} else {
  if ($tag -cnotmatch '\A(?:release/v[0-9]+\.[0-9]+\.[0-9]+|rc/v[0-9]+\.[0-9]+\.[0-9]+-rc\.[0-9]+|nightly/(?:[0-9]{14}\.g[0-9a-f]{12}|latest))\z') {
    Fail "invalid release version: $Version"
  }
  $baseUrl = "https://github.com/$Repo/releases/download/$tag"
}

$manifestUrl = "$baseUrl/atlas-release-manifest.json"
$archive = "(from manifest)"
$installPath = Join-Path $InstallDir "atlas.exe"

function Get-InstallEntry {
  if (!(Test-Path -LiteralPath $InstallDir -PathType Container)) { return }
  # Enumerate the entry itself so a dangling link is visible without following it.
  $entry = Get-ChildItem -LiteralPath $InstallDir -Filter "atlas.exe" -Force |
    Where-Object { $_.Name -eq "atlas.exe" }
  if ($entry.PSIsContainer -or ($entry.Attributes -band [IO.FileAttributes]::Directory)) {
    Fail "install destination is a directory: $installPath"
  }
  return $entry
}

$existingEntry = Get-InstallEntry
$pathEntries = ($env:PATH -split ';') | Where-Object { $_ -ne "" }
$pathStatus = if ($pathEntries -contains $InstallDir) { "on-path" } else { "not-on-path" }
$existing = if ($existingEntry) { "at-destination" } else { "none" }
$existingVersion = "unknown"
if (Test-Path -LiteralPath $installPath -PathType Leaf) {
  try { $existingVersion = & $installPath --version } catch { $existingVersion = "unknown" }
}

$existingVariant = "none"
if ($existingEntry -and !(Test-Path -LiteralPath $installPath -PathType Leaf)) {
  $existingVariant = "legacy"
} elseif ($existingEntry) {
  try { $existingVariant = Get-BuildVariant $installPath }
  catch {
    if (-not ($WithWeb -or $WithoutWeb)) { Fail "could not read build variant from $installPath`; select -WithWeb or -WithoutWeb to replace it" }
    $existingVariant = "unknown"
  }
}
$variant = if ($WithoutWeb) { "cli-only" } elseif ($WithWeb) { "full" }
  elseif ($existingVariant -eq "cli-only") { "cli-only" } else { "full" }

$shadow = $null
$cmd = Get-Command atlas.exe -ErrorAction SilentlyContinue
if ($cmd -and ($cmd.Source -ne $installPath)) { $shadow = $cmd.Source }

Write-Host "Atlas installer"
Write-Host "  target:      $target"
Write-Host "  version:     $Version"
Write-Host "  build:       $variant"
if ($existingVariant -ne "none") { Write-Host "  current build: $existingVariant" }
if ($tag -ceq "nightly/latest") {
  Write-Host "  channel:     $baseUrl/nightly-version.txt (resolved when installing)"
} else {
  Write-Host "  manifest:    $manifestUrl"
}
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
function Resolve-LatestTag {
  try {
    $response = Invoke-WebRequest -Uri "https://github.com/$Repo/releases/latest" `
      -UseBasicParsing -MaximumRedirection 0 -ErrorAction Stop
  } catch {
    # Both PowerShell versions can surface a blocked redirect as a response-bearing error.
    $response = $_.Exception.Response
    if (!$response) { Fail "failed to resolve the latest stable release: $_" }
  }
  $location = if ($response.Headers -is [Net.WebHeaderCollection]) {
    $response.Headers['Location']
  } else {
    $response.Headers.Location
  }
  if ([int]$response.StatusCode -notin @(301, 302, 307, 308) -or !$location) {
    Fail "failed to resolve the latest stable release"
  }
  $tagRoute = "/$Repo/releases/tag/"
  $match = [regex]::Match([string]$location,
    '\A(?:https://github\.com)?' + [regex]::Escape($tagRoute) + '(?<tag>[^\s]+)\z')
  if (!$match.Success) { Fail "no supported latest stable release found; choose a published version" }
  $resolved = $match.Groups['tag'].Value -replace '%2f', '/'
  if ($resolved -cnotmatch '\A(?:release/)?v[0-9]+\.[0-9]+\.[0-9]+\z') {
    Fail "invalid latest stable release tag: $resolved"
  }
  return $resolved
}

try {
  if ($tag -ceq "latest") {
    $tag = Resolve-LatestTag
    Write-Host "  resolved:    $tag"
  } elseif ($tag -ceq "nightly/latest") {
    $pointer = Join-Path $tmp "nightly-version.txt"
    Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/nightly-version.txt" -OutFile $pointer
    $tag = (Get-Content $pointer -Raw).Trim()
    if ($tag -cnotmatch '\Anightly/[0-9]{14}\.g[0-9a-f]{12}\z') {
      Fail "invalid nightly channel pointer"
    }
    Write-Host "  nightly:     $tag"
  }
  $baseUrl = "https://github.com/$Repo/releases/download/$tag"
  $sums = Join-Path $tmp "SHA256SUMS"
  $manifestPath = Join-Path $tmp "atlas-release-manifest.json"
  Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/atlas-release-manifest.json" -OutFile $manifestPath
  $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
  $packageVersion = $manifest.version
  $expectedPackage = ($tag -csplit '/')[-1]
  if ($packageVersion -isnot [string]) { Fail "manifest does not contain a package version" }
  $versionMatches = if ($tag.StartsWith('nightly/')) {
    $packageVersion -cmatch '\Av[0-9]+\.[0-9]+\.[0-9]+-nightly\.[0-9]{14}\.g[0-9a-f]{12}\z' -and
      $packageVersion.EndsWith("-nightly.$expectedPackage", [StringComparison]::Ordinal)
  } else {
    $packageVersion -ceq $expectedPackage
  }
  if (!$versionMatches) {
    Fail "manifest version does not match ${tag}: $packageVersion"
  }
  $expectedVersion = 'atlas ' + $packageVersion.Substring(1)
  Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/SHA256SUMS" -OutFile $sums
  $assets = @($manifest.assets | Where-Object {
    $assetVariant = if ($null -eq $_.variant) { "full" } else { $_.variant }
    $_.target -ceq $target -and $assetVariant -ceq $variant
  })
  if ($assets.Count -ne 1) { Fail "release does not contain exactly one $variant asset for $target; choose a release that provides it" }
  $asset = $assets[0]
  $expectedArchive = if ($variant -eq "full") { "atlas-cli-$target.zip" } else { "atlas-cli-only-$target.zip" }
  if ($asset.name -cne $expectedArchive) { Fail "manifest archive does not match requested build" }
  $archive = $asset.name
  $zip = Join-Path $tmp $archive
  Write-Host "  asset:       $archive"
  Invoke-WebRequest -UseBasicParsing -Uri "$baseUrl/$archive" -OutFile $zip

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
  $newVersion = (& $newBinary.FullName --version) -join "`n"
  if ($LASTEXITCODE -ne 0 -or $newVersion -cne $expectedVersion) {
    Fail "downloaded atlas version mismatch: expected $expectedVersion, got $newVersion"
  }
  Write-Host "  target bin:  $newVersion"
  try { $newVariant = Get-BuildVariant $newBinary.FullName }
  catch { Fail "downloaded atlas has invalid build information: $_" }
  if ($null -eq $asset.variant -and $newVariant -eq "legacy") { $newVariant = "full" }
  if ($newVariant -cne $variant) { Fail "downloaded atlas build does not match requested $variant variant" }

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

  $confirmedEntry = Get-InstallEntry
  if ($confirmedEntry) {
    $change = ClassifyChange $existingVersion $newVersion
    Write-Host "  action:      replace existing atlas ($change)"
    if (-not $Yes) {
      $answer = Read-Host "Replace existing atlas at $installPath? [y/N]"
      if ($answer -notin @("y", "Y", "yes", "YES")) { Fail "install cancelled" }
    }
  }

  $transaction = [Guid]::NewGuid().ToString('N')
  $tmpDest = Join-Path $InstallDir ".atlas.install.$transaction.exe"
  $backup = Join-Path $InstallDir ".atlas.backup.$transaction.exe"
  $movedOld = $false
  $placed = $false
  try {
    Copy-Item -LiteralPath $newBinary.FullName -Destination $tmpDest
    $currentEntry = Get-InstallEntry
    if ($currentEntry) {
      if (!$Yes -and !$confirmedEntry) { throw "atlas.exe appeared during install; retry to confirm replacement" }
      [IO.File]::Move($installPath, $backup)
      $movedOld = $true
    }
    [IO.File]::Move($tmpDest, $installPath)
    $placed = $true
    $installedVersion = (& $installPath --version) -join "`n"
    if ($LASTEXITCODE -ne 0 -or $installedVersion -cne $expectedVersion) {
      throw "installed atlas.exe failed its version check (expected $expectedVersion, got $installedVersion)"
    }
  } catch {
    $failure = $_
    try {
      if ($placed) {
        $installedEntry = Get-InstallEntry
        if ($installedEntry.LinkType -or ($installedEntry -and
            (Get-FileHash -LiteralPath $installPath).Hash -ne (Get-FileHash -LiteralPath $newBinary.FullName).Hash)) {
          throw "atlas.exe changed before it could be removed"
        }
        if ($installedEntry) { [IO.File]::Delete($installPath) }
      }
      if ($movedOld) { [IO.File]::Move($backup, $installPath) }
    } catch {
      $recovery = if ($movedOld) { " Backup: $backup." } else { "" }
      Fail "failed to recover after an installation error: $($_.Exception.Message).$recovery"
    }
    Fail "failed to replace or validate atlas.exe: $($failure.Exception.Message)"
  } finally {
    Remove-Item -LiteralPath $tmpDest -Force -ErrorAction SilentlyContinue
  }
  if ($movedOld) { [IO.File]::Delete($backup) }
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
Write-Host "After a fresh install or Atlas upgrade, run 'atlas setup' to prepare or update reference data and search models."
