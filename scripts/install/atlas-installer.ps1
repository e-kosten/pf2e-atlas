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
  Write-Host "  3. Verify the checksum and extract the entire ZIP to its own folder on PATH"
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
  # Enumerate the entry itself so a dangling link is visible without following it.
  $entry = Find-Entry $InstallDir 'atlas.exe'
  if ($entry.PSIsContainer -or ($entry.Attributes -band [IO.FileAttributes]::Directory)) {
    Fail "install destination is a directory: $installPath"
  }
  return $entry
}

# Only these names may be changed by a Windows CLI install. The receipt binds
# each redistributable to its installed bytes, including manual ZIP installs.
$runtimeNames = @('msvcp140.dll', 'msvcp140_1.dll', 'vcruntime140.dll', 'vcruntime140_1.dll')
$receiptName = 'atlas-windows-runtime.json'
$payloadNames = @($runtimeNames) + @($receiptName, 'atlas.exe')
$pendingDir = Join-Path $InstallDir '.atlas-cli-installing'

function Find-Entry([string]$Directory, [string]$Name) {
  if ([IO.Directory]::Exists($Directory)) {
    Get-ChildItem -Force -LiteralPath $Directory -Filter $Name | Where-Object { $_.Name -eq $Name }
  }
}

function Read-RuntimeReceipt([string]$Directory) {
  $entry = Find-Entry $Directory $receiptName
  if (!$entry) { return }
  if ($entry.PSIsContainer -or ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw "Expected a regular Atlas runtime receipt at $($entry.FullName)"
  }
  $receipt = Get-Content -Raw -LiteralPath $entry.FullName | ConvertFrom-Json
  $names = @($receipt.files.PSObject.Properties.Name)
  if ($receipt.format -cne 'atlas-windows-runtime-v1' -or $names.Count -ne $runtimeNames.Count) {
    throw "Unrecognized Atlas runtime receipt at $($entry.FullName)"
  }
  foreach ($name in $runtimeNames) {
    if ($names -cnotcontains $name -or $receipt.files.$name -notmatch '^[0-9a-f]{64}$') {
      throw "Invalid Atlas runtime receipt for $name at $($entry.FullName)"
    }
  }
  return $receipt.files
}

function Assert-RegularPayload([object]$Entry, [string]$Expected) {
  if (!$Entry -or $Entry.PSIsContainer -or ($Entry.Attributes -band [IO.FileAttributes]::ReparsePoint) -or
      !$Expected -or (Get-FileHash -LiteralPath $Entry.FullName).Hash -ne $Expected) {
    throw "Missing, modified or unowned runtime file: $($Entry.FullName). Use a separate Atlas install folder."
  }
}

function Get-TransactionNames($Journal) {
  $names = @($Journal.files.PSObject.Properties.Name)
  if ($Journal.format -cne 'atlas-cli-install-v1' -or $names -cnotcontains 'atlas.exe' -or
      @($names | Where-Object { $payloadNames -cnotcontains $_ -or $Journal.files.$_ -notmatch '^[0-9a-f]{64}$' }).Count) {
    throw "Unrecognized Atlas install transaction at $pendingDir"
  }
  return $payloadNames | Where-Object { $names -ccontains $_ }
}

function Complete-Transaction([string]$Directory, [string[]]$Names) {
  # Renaming this owned transaction frees the fixed recovery location atomically.
  $garbage = Join-Path $InstallDir ('.atlas-finished-' + [Guid]::NewGuid())
  [IO.Directory]::Move($Directory, $garbage)
  try {
    foreach ($name in $Names) {
      [IO.File]::Delete((Join-Path $garbage "old.$name"))
      [IO.File]::Delete((Join-Path $garbage "new.$name"))
    }
    [IO.File]::Delete((Join-Path $garbage 'transaction.json'))
    [IO.Directory]::Delete($garbage)
  } catch {
    Write-Warning "The transaction finished. Its temporary files remain at $garbage ($($_.Exception.Message))"
  }
}

function Restore-Transaction {
  $entry = Find-Entry $InstallDir '.atlas-cli-installing'
  if (!$entry) { return }
  if (!$entry.PSIsContainer -or ($entry.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
    throw "Unexpected installation recovery path: $pendingDir"
  }
  $journal = Get-Content -LiteralPath (Join-Path $pendingDir 'transaction.json') -Raw | ConvertFrom-Json
  $names = @(Get-TransactionNames $journal)
  # Withdraw a newly placed executable first. Keep new.* as an idempotent
  # record of what is already withdrawn if recovery itself is interrupted.
  foreach ($name in @('atlas.exe') + @($names | Where-Object { $_ -ne 'atlas.exe' })) {
    if (!(Find-Entry $pendingDir "new.$name")) {
      $installed = Find-Entry $InstallDir $name
      if ($installed) {
        try { Assert-RegularPayload $installed $journal.files.$name }
        catch { throw "Cannot undo $($installed.FullName) because it changed." }
        [IO.File]::Move($installed.FullName, (Join-Path $pendingDir "new.$name"))
      } else {
        # A removed candidate needs the same withdrawal marker so a later
        # recovery will recognize an already restored original.
        [IO.File]::WriteAllBytes((Join-Path $pendingDir "new.$name"), [byte[]]@())
      }
    }
  }
  # Restore the original executable last, after all its runtime files.
  foreach ($name in $names) {
    $backup = Find-Entry $pendingDir "old.$name"
    if ($backup) { [IO.File]::Move($backup.FullName, (Join-Path $InstallDir $name)) }
  }
  Complete-Transaction $pendingDir $names
}

# A file in the destination serializes all sessions and directory aliases.
# CreateNew refuses any existing file or link. DeleteOnClose releases only
# our newly created file, including when the installer process terminates.
$lockName = '.atlas-cli-install.lock'
$lockPath = Join-Path $InstallDir $lockName
$lease = $null
try {
if (Find-Entry $InstallDir $lockName) { Fail "Another installer or an existing entry occupies $lockPath. Use a separate Atlas install folder if the entry is unrelated." }
if (!$DryRun) {
  [IO.Directory]::CreateDirectory($InstallDir) | Out-Null
  try {
    $lease = [IO.FileStream]::new($lockPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::ReadWrite,
      [IO.FileShare]::None, 4096, [IO.FileOptions]::DeleteOnClose)
  } catch [IO.IOException] { Fail "Could not acquire the Atlas installation lock at $lockPath. Retry after the other installer exits, or choose another directory." }
  try { Restore-Transaction } catch {
    Write-Host "Recovery files: $pendingDir"
    Fail "Could not recover the interrupted installation: $($_.Exception.Message)"
  }
}
$existingEntry = Get-InstallEntry
$pathEntries = ($env:PATH -split ';') | Where-Object { $_ -ne "" }
$pathStatus = if ($pathEntries -contains $InstallDir) { "on-path" } else { "not-on-path" }
$existing = if ($existingEntry) { "at-destination" } else { "none" }
$existingVersion = "unknown"
if ($existingEntry) {
  try {
    $reportedVersion = (& $installPath --version) -join "`n"
    if ($LASTEXITCODE -eq 0 -and $reportedVersion) { $existingVersion = $reportedVersion }
  } catch { $existingVersion = "unknown" }
}

$existingVariant = "none"
# Windows can report a dangling symlink as a leaf. The version probe determines whether it runs.
if ($existingEntry.LinkType -eq "SymbolicLink" -and $existingVersion -eq "unknown") {
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
  $binaries = @(Get-ChildItem -LiteralPath $extract -Filter atlas.exe -Recurse -File)
  if ($binaries.Count -ne 1) { Fail "expected one atlas.exe in the archive" }
  $newBinary = $binaries[0]
  $newRuntime = Read-RuntimeReceipt $newBinary.DirectoryName
  $oldRuntime = Read-RuntimeReceipt $InstallDir
  if ($oldRuntime) { $oldReceiptHash = (Get-FileHash -LiteralPath (Join-Path $InstallDir $receiptName)).Hash }
  foreach ($name in $runtimeNames) {
    $packaged = Find-Entry $newBinary.DirectoryName $name
    $installed = Find-Entry $InstallDir $name
    if ($packaged -or $newRuntime) { Assert-RegularPayload $packaged $newRuntime.$name }
    if ($installed -and ($newRuntime -or $oldRuntime)) {
      Assert-RegularPayload $installed $oldRuntime.$name
      if (!$newRuntime) {
        # Older archives require the same runtime but do not ship it. Retain
        # our installed runtime and use it for the downloaded version probe.
        Copy-Item -LiteralPath $installed.FullName -Destination $newBinary.DirectoryName
      }
    }
  }
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

  # Stage the complete verified payload on the destination volume before retiring
  # the old executable. Once active, the journal survives an interrupted shell.
  $stage = Join-Path $InstallDir ('.atlas-staging-' + [Guid]::NewGuid())
  [IO.Directory]::CreateDirectory($stage) | Out-Null
  $names = @('atlas.exe')
  if ($newRuntime) { $names = $payloadNames }
  $ownsPending = $false
  try {
    $hashes = [ordered]@{}
    foreach ($name in $names) {
      $source = Join-Path $newBinary.DirectoryName $name
      $expectedHash = if ($runtimeNames -contains $name) { $newRuntime.$name } else { (Get-FileHash -LiteralPath $source).Hash }
      $staged = Join-Path $stage "new.$name"
      Copy-Item -LiteralPath $source -Destination $staged
      $hashes[$name] = (Get-FileHash -LiteralPath $staged).Hash
      if ($hashes[$name] -ne $expectedHash) { throw "Staged $name did not match the verified archive" }
    }
    @{ format = 'atlas-cli-install-v1'; files = $hashes } | ConvertTo-Json |
      Set-Content -LiteralPath (Join-Path $stage 'transaction.json') -Encoding ascii

    # An executable can be a symlink, including a dangling one. Move the entry
    # itself as before. Runtime files require a matching ownership receipt.
    $currentEntry = Get-InstallEntry
    if ($currentEntry -and !$Yes -and !$confirmedEntry) { throw "atlas.exe appeared during install; retry to confirm replacement" }
    [IO.Directory]::Move($stage, $pendingDir)
    $ownsPending = $true
    if ($currentEntry) { [IO.File]::Move($installPath, (Join-Path $pendingDir 'old.atlas.exe')) }
    foreach ($name in $names) {
      $targetPath = Join-Path $InstallDir $name
      $current = Find-Entry $InstallDir $name
      if ($current) {
        if ($name -eq 'atlas.exe') { throw "atlas.exe appeared during replacement" }
        $expected = if ($name -eq $receiptName) { $oldReceiptHash } else { $oldRuntime.$name }
        Assert-RegularPayload $current $expected
        [IO.File]::Move($targetPath, (Join-Path $pendingDir "old.$name"))
      }
      [IO.File]::Move((Join-Path $pendingDir "new.$name"), $targetPath)
    }
    $installedVersion = (& $installPath --version) -join "`n"
    if ($LASTEXITCODE -ne 0 -or $installedVersion -cne $expectedVersion) {
      throw "installed atlas.exe failed its version check (expected $expectedVersion, got $installedVersion)"
    }
    Complete-Transaction $pendingDir $names
  } catch {
    $failure = $_
    if ($ownsPending) {
      try { Restore-Transaction }
      catch {
        Write-Host "Recovery files: $pendingDir"
        Fail "failed to recover after an installation error: $($_.Exception.Message)"
      }
    }
    Fail "failed to replace or validate Atlas: $($failure.Exception.Message)"
  } finally {
    if ([IO.Directory]::Exists($stage)) {
      foreach ($name in $names) { [IO.File]::Delete((Join-Path $stage "new.$name")) }
      [IO.File]::Delete((Join-Path $stage 'transaction.json'))
      [IO.Directory]::Delete($stage)
    }
  }
} finally {
  Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "Installed atlas to $installPath"
$setupHint = "run 'atlas setup'"
if ($InstallDir.Contains(';') -or $InstallDir -match '%[^%]+%') {
  # Windows PowerShell and PowerShell 7 both split PATH at semicolons,
  # including those inside quoted directory names.
  $quotedExecutable = [Management.Automation.Language.CodeGeneration]::EscapeSingleQuotedStringContent($installPath)
  $reason = if ($InstallDir.Contains(';')) { 'PowerShell splits PATH at semicolons.' } else { 'Windows can expand %variable% references in your user PATH.' }
  Write-Host "$reason Run Atlas in PowerShell using its full path:"
  Write-Host "  & '$quotedExecutable' setup"
  $setupHint = 'run the command above'
} elseif ($pathStatus -ne "on-path") {
  Write-Host ""
  Write-Host "$InstallDir is not on PATH. Add it for this PowerShell session and future sign-ins."
  Write-Host "PowerShell hint:"
  $quotedDirectory = [Management.Automation.Language.CodeGeneration]::EscapeSingleQuotedStringContent($InstallDir)
  Write-Host "  `$atlasBin = '$quotedDirectory'"
  Write-Host @'
  $key = [Microsoft.Win32.Registry]::CurrentUser.CreateSubKey('Environment')
  try {
    if (($key.GetValue('Path', '') -split ';') -notcontains $atlasBin) {
      $userPath = $key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
      $kind = if ($key.GetValueNames() -contains 'Path') { $key.GetValueKind('Path') } else { 'String' }
      $key.SetValue('Path', ((@($userPath, $atlasBin) | Where-Object { $_ }) -join ';'), $kind)
    }
  } finally { $key.Dispose() }
  if (($env:Path -split ';') -notcontains $atlasBin) { $env:Path += ";$atlasBin" }
'@
  Write-Host "Other apps inherit the updated PATH after your next Windows sign-in."
}
Write-Host "After a fresh install or Atlas upgrade, $setupHint to prepare or update reference data and search models."
} finally {
  if ($lease) { $lease.Dispose() }
}
