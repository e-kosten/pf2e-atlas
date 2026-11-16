param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path,
  [string]$Shell = (Get-Process -Id $PID).Path
)

$ErrorActionPreference = "Stop"
if ($PSVersionTable.PSVersion.Major -ge 7) {
  $PSNativeCommandUseErrorActionPreference = $true
}
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("atlas release test-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
  $dist = Join-Path $tmp "dist"
  New-Item -ItemType Directory -Force -Path $dist | Out-Null
  $notices = Join-Path $dist "THIRD-PARTY-NOTICES.md"
  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/generate-notices.js") --output $notices | Out-Null

  function New-UnixArchive($Target) {
    $work = Join-Path $tmp "work"
    $root = Join-Path $work "atlas-cli-$Target"
    $archive = Join-Path $dist "atlas-cli-$Target.tar.xz"
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    "#!/bin/sh`nprintf 'atlas 9.9.9\n'`n" | Set-Content -Path (Join-Path $root "atlas") -NoNewline
    Copy-Item (Join-Path $RepoRoot "LICENSE") (Join-Path $root "LICENSE")
    Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $root "README.md")
    Copy-Item $notices (Join-Path $root "THIRD-PARTY-NOTICES.md")
    node (Join-Path $RepoRoot "dev-tools/release/dist/tests/release-fixtures.js") unix-archive $archive $root
  }

  function New-WindowsArchive($Target, $Variant = "full", $Destination = $dist) {
    $prefix = if ($Variant -eq "cli-only") { "atlas-cli-only" } else { "atlas-cli" }
    $root = Join-Path $tmp "$prefix-$Target-$Variant"
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    $appDir = Join-Path $tmp "stub-app-$Variant"
    if ($env:OS -ne "Windows_NT") {
      @'
#!/bin/sh
version=${ATLAS_TEST_BINARY_VERSION:-9.9.9}
if [ '__VARIANT__' = rollback ]; then
  case "$0" in */extract/*) ;; *) version=9.9.10 ;; esac
fi
case "$1" in
  --version) printf 'atlas %s\n' "$version" ;;
  build-info)
    if [ '__VARIANT__' = legacy ]; then
      printf 'unknown subcommand build-info\n' >&2
      exit 2
    fi
    printf '{"status":"ok","data":{"version":"9.9.9","variant":"__VARIANT__"}}\n' ;;
  *) exit 2 ;;
esac
'@.Replace('__VARIANT__', $Variant).Replace('variant":"rollback"', 'variant":"full"') | Set-Content -Path (Join-Path $root "atlas.exe") -NoNewline
      chmod +x (Join-Path $root "atlas.exe")
    } else {
    if (-not (Test-Path (Join-Path $appDir "bin"))) {
      New-Item -ItemType Directory -Force -Path $appDir | Out-Null
      @'
<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
  </PropertyGroup>
</Project>
'@ | Set-Content -Path (Join-Path $appDir "atlas-stub.csproj")
      @'
var version = Environment.GetEnvironmentVariable("ATLAS_TEST_BINARY_VERSION") ?? "9.9.9";
if ("__VARIANT__" == "rollback" && !AppContext.BaseDirectory.Contains(@"\extract\", StringComparison.OrdinalIgnoreCase)) {
    version = "9.9.10";
}
if (args.Length > 0 && args[0] == "--version") { Console.WriteLine($"atlas {version}"); }
else if (args.Length > 0 && args[0] == "build-info" && "__VARIANT__" != "legacy") {
  Console.WriteLine("{\"status\":\"ok\",\"data\":{\"version\":\"9.9.9\",\"variant\":\"__VARIANT__\"}}");
} else { Console.Error.WriteLine("unknown subcommand build-info"); Environment.Exit(2); }
'@.Replace('__VARIANT__', $Variant).Replace('variant\":\"rollback\"', 'variant\":\"full\"') | Set-Content -Path (Join-Path $appDir "Program.cs")
      dotnet publish $appDir --configuration Release --runtime win-x64 --self-contained true -p:PublishSingleFile=true -p:PublishTrimmed=true --output (Join-Path $appDir "out") | Out-Null
    }
    Copy-Item (Join-Path $appDir "out/atlas-stub.exe") (Join-Path $root "atlas.exe")
    }
    Copy-Item (Join-Path $RepoRoot "LICENSE") (Join-Path $root "LICENSE")
    Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $root "README.md")
    Copy-Item $notices (Join-Path $root "THIRD-PARTY-NOTICES.md")
    Compress-Archive -Path $root -DestinationPath (Join-Path $Destination "$prefix-$Target.zip") -Force
  }

  @(
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu"
  ) | ForEach-Object {
    New-UnixArchive $_
    Copy-Item (Join-Path $dist "atlas-cli-$_.tar.xz") (Join-Path $dist "atlas-cli-only-$_.tar.xz")
  }

  @(
    "x86_64-pc-windows-msvc"
  ) | ForEach-Object { New-WindowsArchive $_ full; New-WindowsArchive $_ cli-only }

  Copy-Item (Join-Path $RepoRoot "scripts/install/atlas-installer.sh") (Join-Path $dist "atlas-installer.sh")
  Copy-Item (Join-Path $RepoRoot "scripts/install/atlas-installer.ps1") (Join-Path $dist "atlas-installer.ps1")
  "{}" | Set-Content -Path (Join-Path $dist "dist-manifest.json")
  "desktop image fixture" | Set-Content -Path (Join-Path $dist "atlas-desktop-aarch64-apple-darwin.dmg")
  "desktop installer fixture" | Set-Content -Path (Join-Path $dist "atlas-desktop-x86_64-pc-windows-msvc-setup.exe")

  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/generate-release-manifest.js") v9.9.9 $dist
  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/validate-release-assets.js") v9.9.9 $dist

  $badManifestDist = Join-Path $tmp "bad-manifest-dist"
  Copy-Item $dist $badManifestDist -Recurse
  $manifestPath = Join-Path $badManifestDist "atlas-release-manifest.json"
  $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
  $manifest.assets = @($manifest.assets | Where-Object { $_.target -ne "aarch64-apple-darwin" })
  $manifest | ConvertTo-Json -Depth 5 | Set-Content $manifestPath
  $acceptedBadManifest = $true
  try {
    node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/validate-release-assets.js") v9.9.9 $badManifestDist | Out-Null
    $acceptedBadManifest = $LASTEXITCODE -eq 0
  } catch {
    $acceptedBadManifest = $false
  }
  if ($acceptedBadManifest) {
    throw "asset validation accepted a manifest missing a required target"
  }

  $fakeBin = Join-Path $tmp "bin"
  $installDir = Join-Path $tmp "install with spaces"
  New-Item -ItemType Directory -Force -Path $fakeBin, $installDir | Out-Null
  @'
param(
  [string]$Uri,
  [string]$OutFile,
  [switch]$UseBasicParsing,
  [int]$MaximumRedirection,
  [string]$ErrorAction
)
if ($env:ATLAS_URL_LOG) { Add-Content -Path $env:ATLAS_URL_LOG -Value $Uri }
if ($Uri.EndsWith('/releases/latest')) {
  if ($MaximumRedirection -ne 0 -or !$UseBasicParsing) { throw "redirect handling is not explicit" }
  $tag = if (@(Get-Content $env:ATLAS_URL_LOG | Where-Object { $_.EndsWith('/releases/latest') }).Count -eq 1) {
    "/e-kosten/pf2e-atlas/releases/tag/release%2Fv9.9.9"
  } else {
    "/e-kosten/pf2e-atlas/releases/tag/release/v9.9.10"
  }
  return @{ StatusCode = 302; Headers = @{ Location = $tag } }
}
if ($Uri.Contains('/releases/latest/download/')) { throw "download requested from a rolling URL" }
$name = Split-Path ([System.Uri]$Uri).AbsolutePath -Leaf
Copy-Item (Join-Path $env:ATLAS_FAKE_RELEASE_DIR $name) $OutFile
'@ | Set-Content -Path (Join-Path $fakeBin "Invoke-WebRequest.ps1")

  @'
function Invoke-WebRequest {
  & (Join-Path $env:ATLAS_FAKE_BIN "Invoke-WebRequest.ps1") @args
}
function Expand-Archive {
  param([string]$Path, [string]$DestinationPath)
  Microsoft.PowerShell.Archive\Expand-Archive -Path $Path -DestinationPath $DestinationPath
  if ($env:OS -ne "Windows_NT") { Get-ChildItem $DestinationPath -Filter atlas.exe -Recurse | ForEach-Object { chmod +x $_.FullName } }
}
function Get-FileHash {
  param([string]$Path, [string]$LiteralPath, [string]$Algorithm)
  $installed = Join-Path $env:ATLAS_TEST_INSTALL_DIR "atlas.exe"
  if ($env:ATLAS_TEST_OBSTRUCT_RESTORE -and $LiteralPath -eq $installed) {
    [IO.File]::WriteAllText($installed, "intervening file")
  }
  Microsoft.PowerShell.Utility\Get-FileHash @PSBoundParameters
}
if ($env:ATLAS_TEST_INSTALL_CWD) { Set-Location -LiteralPath $env:ATLAS_TEST_INSTALL_CWD }
$variantArgs = @{}
if ($env:ATLAS_TEST_VARIANT -eq "cli-only") { $variantArgs.WithoutWeb = $true }
if ($env:ATLAS_TEST_VARIANT -eq "full") { $variantArgs.WithWeb = $true }
$version = if ($env:ATLAS_TEST_INSTALL_VERSION) { $env:ATLAS_TEST_INSTALL_VERSION } else { 'v9.9.9' }
& $env:ATLAS_INSTALLER -InstallDir $env:ATLAS_TEST_INSTALL_DIR -Version $version -Yes @variantArgs
'@ | Set-Content -Path (Join-Path $fakeBin "run-install.ps1")

  $env:ATLAS_FAKE_BIN = $fakeBin
  $env:ATLAS_FAKE_RELEASE_DIR = $dist
  $env:ATLAS_INSTALLER = Join-Path $RepoRoot "scripts/install/atlas-installer.ps1"
  if ($env:OS -ne "Windows_NT") {
    # Exercise the native installer logic locally with executable script fixtures.
    $portableInstaller = Join-Path $tmp "atlas-installer.ps1"
    (Get-Content $env:ATLAS_INSTALLER -Raw).Replace('[System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()', '"X64"') |
      Set-Content $portableInstaller
    $env:ATLAS_INSTALLER = $portableInstaller
  }
  $env:ATLAS_TEST_INSTALL_DIR = $installDir
  $dryRunDir = Join-Path $tmp "dry-run-install"
  & $env:ATLAS_INSTALLER -InstallDir $dryRunDir -Version v9.9.9 -DryRun | Out-Null
  if (Test-Path (Join-Path $dryRunDir "atlas.exe")) {
    throw "PowerShell installer dry-run wrote an atlas.exe binary"
  }

  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null

  $installedVersion = & (Join-Path $installDir "atlas.exe") --version
  if ($installedVersion.Trim() -ne "atlas 9.9.9") {
    throw "unexpected installed version: $installedVersion"
  }

  $env:ATLAS_URL_LOG = Join-Path $tmp "latest-urls"
  $env:ATLAS_TEST_INSTALL_VERSION = "latest"
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "PowerShell latest install failed" }
  $latestUrls = @(Get-Content $env:ATLAS_URL_LOG)
  if ($latestUrls.Count -ne 4 -or $latestUrls[0] -ne "https://github.com/e-kosten/pf2e-atlas/releases/latest" -or
      @($latestUrls | Select-Object -Skip 1 | Where-Object { $_ -notlike '*/releases/download/release/v9.9.9/*' }).Count) {
    throw "PowerShell latest install did not pin its download URLs"
  }
  Remove-Item Env:ATLAS_URL_LOG, Env:ATLAS_TEST_INSTALL_VERSION

  $env:ATLAS_TEST_INSTALL_CWD = $tmp
  $env:ATLAS_TEST_INSTALL_DIR = "relative install"
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  if ($LASTEXITCODE -ne 0 -or !(Test-Path -LiteralPath (Join-Path $tmp "relative install/atlas.exe") -PathType Leaf)) {
    throw "PowerShell installer did not resolve a relative path from the current location"
  }
  Remove-Item Env:ATLAS_TEST_INSTALL_CWD
  $env:ATLAS_TEST_INSTALL_DIR = $installDir

  $secondOutput = & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1")
  if (($secondOutput -join "`n") -notmatch [regex]::Escape("not on PATH")) {
    throw "PowerShell installer did not print PATH guidance"
  }
  if (($secondOutput -join "`n") -notmatch [regex]::Escape("After a fresh install or Atlas upgrade, run 'atlas setup'")) {
    throw "PowerShell installer did not print setup guidance"
  }

  foreach ($choice in @("cli-only", "preserve-cli", "full", "preserve-full")) {
    $env:ATLAS_TEST_VARIANT = if ($choice.StartsWith("preserve")) { "" } else { $choice }
    & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
    $info = (& (Join-Path $installDir "atlas.exe") build-info --json) | ConvertFrom-Json
    $expected = if ($choice -in @("cli-only", "preserve-cli")) { "cli-only" } else { "full" }
    if ($info.data.variant -ne $expected) { throw "PowerShell installer failed variant selection: $choice" }
  }
  Remove-Item Env:ATLAS_TEST_VARIANT
  $failed = $false
  try { & $Shell -NoProfile -File $env:ATLAS_INSTALLER -InstallDir $installDir -WithWeb -WithoutWeb -DryRun | Out-Null; $failed = $LASTEXITCODE -ne 0 } catch { $failed = $true }
  if (-not $failed) { throw "PowerShell installer accepted conflicting web choices" }

  "invalid existing executable" | Set-Content (Join-Path $installDir "atlas.exe")
  $env:ATLAS_TEST_VARIANT = "cli-only"
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  $info = (& (Join-Path $installDir "atlas.exe") build-info --json) | ConvertFrom-Json
  if ($info.data.variant -ne "cli-only") { throw "PowerShell explicit selection could not repair an invalid destination" }
  $env:ATLAS_TEST_VARIANT = "full"
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  Remove-Item Env:ATLAS_TEST_VARIANT

  $legacyDist = Join-Path $tmp "legacy-dist"
  Copy-Item $dist $legacyDist -Recurse
  New-WindowsArchive "x86_64-pc-windows-msvc" legacy $legacyDist
  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/generate-release-manifest.js") v9.9.9 $legacyDist
  node (Join-Path $RepoRoot "dev-tools/release/dist/tests/release-fixtures.js") legacy-manifest (Join-Path $legacyDist "atlas-release-manifest.json")
  $env:ATLAS_FAKE_RELEASE_DIR = $legacyDist
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  $env:ATLAS_TEST_VARIANT = "cli-only"
  $failed = $false
  try { & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null; $failed = $LASTEXITCODE -ne 0 } catch { $failed = $true }
  if (-not $failed) { throw "PowerShell installer substituted full for unavailable CLI-only" }
  Remove-Item Env:ATLAS_TEST_VARIANT
  $env:ATLAS_FAKE_RELEASE_DIR = $dist
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null

  @'
function Invoke-WebRequest {
  & (Join-Path $env:ATLAS_FAKE_BIN "Invoke-WebRequest.ps1") @args
}
function Expand-Archive {
  param([string]$Path, [string]$DestinationPath)
  Microsoft.PowerShell.Archive\Expand-Archive -Path $Path -DestinationPath $DestinationPath
  if ($env:OS -ne "Windows_NT") { Get-ChildItem $DestinationPath -Filter atlas.exe -Recurse | ForEach-Object { chmod +x $_.FullName } }
}
& $env:ATLAS_INSTALLER -InstallDir $env:ATLAS_TEST_INSTALL_DIR -Version v9.9.9
'@ | Set-Content -Path (Join-Path $fakeBin "run-install-no-yes.ps1")
  $failed = $false
  try {
    & $Shell -NoProfile -File (Join-Path $fakeBin "run-install-no-yes.ps1") | Out-Null
    $failed = $LASTEXITCODE -ne 0
  } catch {
    $failed = $true
  }
  if (-not $failed) {
    throw "PowerShell installer replaced existing binary without -Yes"
  }

  $nightlyTag = "nightly/20261018092300.gabcdef123456"
  $nightlyVersion = "9.9.9-nightly." + $nightlyTag.Split('/')[-1]
  $nightlyDist = Join-Path $tmp "nightly-dist"
  Copy-Item $dist $nightlyDist -Recurse
  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/generate-release-manifest.js") "v$nightlyVersion" $nightlyDist
  Set-Content -Path (Join-Path $nightlyDist "nightly-version.txt") -Value $nightlyTag
  $env:ATLAS_FAKE_RELEASE_DIR = $nightlyDist
  $env:ATLAS_TEST_INSTALL_DIR = Join-Path $tmp "nightly-install"
  $env:ATLAS_TEST_INSTALL_VERSION = "nightly"
  $env:ATLAS_TEST_BINARY_VERSION = $nightlyVersion
  $env:ATLAS_URL_LOG = Join-Path $tmp "nightly-urls"
  & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "PowerShell nightly install failed" }
  $urls = @(Get-Content $env:ATLAS_URL_LOG)
  $expectedUrls = @(
    "https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/latest/nightly-version.txt"
    "https://github.com/e-kosten/pf2e-atlas/releases/download/$nightlyTag/atlas-release-manifest.json"
    "https://github.com/e-kosten/pf2e-atlas/releases/download/$nightlyTag/SHA256SUMS"
    "https://github.com/e-kosten/pf2e-atlas/releases/download/$nightlyTag/atlas-cli-x86_64-pc-windows-msvc.zip"
  )
  if (($urls -join "`n") -ne ($expectedUrls -join "`n")) {
    throw "nightly installer requested unexpected release assets"
  }
  Remove-Item Env:ATLAS_URL_LOG
  foreach ($badTag in @("v9.9.9", "nightly/20261018092400.gabcdef123456", "nightly/20261018092300.gabcdef123457")) {
    Set-Content -Path (Join-Path $nightlyDist "nightly-version.txt") -Value $badTag
    $failed = $false
    try {
      & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
      $failed = $LASTEXITCODE -ne 0
    } catch { $failed = $true }
    if (!$failed) { throw "nightly installer accepted a mismatched channel pointer: $badTag" }
  }
  Remove-Item Env:ATLAS_TEST_INSTALL_VERSION, Env:ATLAS_TEST_BINARY_VERSION
  $env:ATLAS_TEST_INSTALL_DIR = $installDir

  $identityDist = Join-Path $tmp "identity-dist"
  Copy-Item $dist $identityDist -Recurse
  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/generate-release-manifest.js") v1.2.3 $identityDist
  $env:ATLAS_FAKE_RELEASE_DIR = $identityDist
  $before = (Get-FileHash -LiteralPath (Join-Path $installDir "atlas.exe")).Hash
  foreach ($version in @("v9.9.9", "v1.2.3")) {
    $env:ATLAS_TEST_INSTALL_VERSION = $version
    $previousNativePreference = $PSNativeCommandUseErrorActionPreference
    $previousErrorPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $PSNativeCommandUseErrorActionPreference = $false
    try {
      $output = & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") 2>&1 | Out-String
      $message = if ($version -eq "v9.9.9") { "manifest version does not match" } else { "downloaded atlas version mismatch" }
      if ($LASTEXITCODE -ne 1 -or $output -notmatch $message) {
        throw "PowerShell installer did not reject an identity mismatch: $output"
      }
    } finally {
      $ErrorActionPreference = $previousErrorPreference
      $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    }
    if ((Get-FileHash -LiteralPath (Join-Path $installDir "atlas.exe")).Hash -ne $before) {
      throw "PowerShell installer changed the prior binary on an identity mismatch"
    }
  }
  Remove-Item Env:ATLAS_TEST_INSTALL_VERSION

  $directoryInstall = Join-Path $tmp "directory-install"
  $linkedInstall = Join-Path $tmp "link-install"
  $linkedTarget = Join-Path $tmp "linked-directory"
  New-Item -ItemType Directory -Path (Join-Path $directoryInstall "atlas.exe"), $linkedInstall, $linkedTarget | Out-Null
  $linkType = if ($env:OS -eq "Windows_NT") { "Junction" } else { "SymbolicLink" }
  $link = Join-Path $linkedInstall "atlas.exe"
  New-Item -ItemType $linkType -Path $link -Target $linkedTarget | Out-Null
  try {
    foreach ($destination in @($directoryInstall, $linkedInstall)) {
      $env:ATLAS_TEST_INSTALL_DIR = $destination
      $env:ATLAS_URL_LOG = Join-Path $tmp "directory-urls"
      $previousNativePreference = $PSNativeCommandUseErrorActionPreference
      $previousErrorPreference = $ErrorActionPreference
      $ErrorActionPreference = "Continue"
      $PSNativeCommandUseErrorActionPreference = $false
      try {
        $output = & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") 2>&1 | Out-String
        if ($LASTEXITCODE -ne 1 -or $output -notmatch 'install destination is a directory' -or
            (Test-Path $env:ATLAS_URL_LOG) -or @(Get-ChildItem -Force -LiteralPath (Join-Path $destination "atlas.exe")).Count) {
          throw "PowerShell installer changed a directory destination: $output"
        }
      } finally {
        $ErrorActionPreference = $previousErrorPreference
        $PSNativeCommandUseErrorActionPreference = $previousNativePreference
      }
    }
  } finally {
    [IO.Directory]::Delete($link)
  }
  Remove-Item Env:ATLAS_URL_LOG
  $env:ATLAS_TEST_INSTALL_DIR = $installDir

  $rollbackDist = Join-Path $tmp "rollback-dist"
  Copy-Item $dist $rollbackDist -Recurse
  $archive = "atlas-cli-x86_64-pc-windows-msvc.zip"
  New-WindowsArchive "x86_64-pc-windows-msvc" rollback $rollbackDist
  node (Join-Path $RepoRoot "dev-tools/release/dist/src/cli/generate-release-manifest.js") v9.9.9 $rollbackDist
  $env:ATLAS_FAKE_RELEASE_DIR = $rollbackDist
  $freshDir = Join-Path $tmp "fresh-rollback"
  $previousPath = Join-Path $installDir "atlas.exe"
  $previousHash = (Get-FileHash $previousPath).Hash
  $fileLinkDir = Join-Path $tmp "file-link-rollback"
  $danglingLinkDir = Join-Path $tmp "dangling-link-rollback"
  New-Item -ItemType Directory -Path $fileLinkDir, $danglingLinkDir | Out-Null
  $missingTarget = Join-Path $tmp "missing-target.exe"
  New-Item -ItemType SymbolicLink -Path (Join-Path $fileLinkDir "atlas.exe") -Target $previousPath | Out-Null
  New-Item -ItemType SymbolicLink -Path (Join-Path $danglingLinkDir "atlas.exe") -Target $missingTarget | Out-Null
  foreach ($destination in @($installDir, $freshDir, $fileLinkDir, $danglingLinkDir)) {
    $env:ATLAS_TEST_INSTALL_DIR = $destination
    $previousNativePreference = $PSNativeCommandUseErrorActionPreference
    $previousErrorPreference = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    $PSNativeCommandUseErrorActionPreference = $false
    try {
      $output = & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") 2>&1 | Out-String
      if ($LASTEXITCODE -ne 1 -or $output -notmatch 'Manual install fallback' -or
          $output -notmatch 'installed atlas\.exe failed its' -or $output -match 'Installed atlas to') {
        throw "PowerShell installer did not report post-move validation failure: $output"
      }
    } finally {
      $ErrorActionPreference = $previousErrorPreference
      $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    }
  }
  $env:ATLAS_TEST_INSTALL_DIR = $installDir
  $installedVersion = & (Join-Path $installDir "atlas.exe") --version
  if ($installedVersion.Trim() -ne "atlas 9.9.9" -or
      (Get-FileHash (Join-Path $installDir "atlas.exe")).Hash -ne $previousHash -or
      (Test-Path (Join-Path $freshDir "atlas.exe"))) {
    throw "PowerShell installer left a failed executable after validation failure"
  }

  $fileLink = Get-ChildItem -Force -LiteralPath $fileLinkDir -Filter "atlas.exe"
  $danglingLink = Get-ChildItem -Force -LiteralPath $danglingLinkDir -Filter "atlas.exe"
  if ($fileLink.LinkType -ne "SymbolicLink" -or $fileLink.Target -ne $previousPath -or
      $danglingLink.LinkType -ne "SymbolicLink" -or $danglingLink.Target -ne $missingTarget) {
    throw "PowerShell installer failed to restore the prior symlink entries"
  }

  $obstructedDir = Join-Path $tmp "obstructed-restore"
  New-Item -ItemType Directory -Path $obstructedDir | Out-Null
  Copy-Item -LiteralPath $previousPath -Destination (Join-Path $obstructedDir "atlas.exe")
  $env:ATLAS_TEST_INSTALL_DIR = $obstructedDir
  $env:ATLAS_TEST_OBSTRUCT_RESTORE = "1"
  $previousNativePreference = $PSNativeCommandUseErrorActionPreference
  $previousErrorPreference = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  $PSNativeCommandUseErrorActionPreference = $false
  try {
    $output = & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") 2>&1 | Out-String
    $backups = @(Get-ChildItem -LiteralPath $obstructedDir -Filter ".atlas.backup.*.exe" -Force)
    if ($LASTEXITCODE -ne 1 -or $backups.Count -ne 1 -or $output -notmatch [regex]::Escape($backups[0].Name) -or
        (Get-FileHash -LiteralPath $backups[0].FullName).Hash -ne $previousHash -or
        (Get-Content -LiteralPath (Join-Path $obstructedDir "atlas.exe") -Raw) -ne "intervening file") {
      throw "PowerShell installer did not preserve the backup and intervening entry: $output"
    }
  } finally {
    $ErrorActionPreference = $previousErrorPreference
    $PSNativeCommandUseErrorActionPreference = $previousNativePreference
    Remove-Item Env:ATLAS_TEST_OBSTRUCT_RESTORE
  }
  $env:ATLAS_TEST_INSTALL_DIR = $installDir

  $badDist = Join-Path $tmp "bad-dist"
  Copy-Item $dist $badDist -Recurse
  $lines = Get-Content (Join-Path $badDist "SHA256SUMS")
  $lines = $lines | ForEach-Object {
    $parts = $_ -split '\s+', 2
    if ($parts.Count -eq 2 -and $parts[1] -eq $archive) {
      "0000000000000000000000000000000000000000000000000000000000000000  $archive"
    } else {
      $_
    }
  }
  Set-Content -Path (Join-Path $badDist "SHA256SUMS") -Value $lines
  $env:ATLAS_FAKE_RELEASE_DIR = $badDist
  $failed = $false
  try {
    & $Shell -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
    $failed = $LASTEXITCODE -ne 0
  } catch {
    $failed = $true
  }
  if (-not $failed) {
    throw "installer accepted a checksum mismatch"
  }

  Write-Host "PowerShell installer smoke tests passed ($Shell)"
  if ($env:OS -eq "Windows_NT" -and $PSVersionTable.PSEdition -eq "Core" -and !$PSBoundParameters.ContainsKey("Shell")) {
    # The Windows gate runs here with PowerShell 7. Repeat with Desktop-edition children.
    & $Shell -NoProfile -File $PSCommandPath -RepoRoot $RepoRoot -Shell powershell.exe
    if ($LASTEXITCODE -ne 0) { throw "Windows PowerShell installer smoke tests failed" }
  }
  exit 0
} finally {
  Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
