param(
  [string]$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "../..")).Path
)

$ErrorActionPreference = "Stop"
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("atlas-release-test-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

try {
  $dist = Join-Path $tmp "dist"
  New-Item -ItemType Directory -Force -Path $dist | Out-Null

  function New-UnixArchive($Target) {
    $work = Join-Path $tmp "work"
    $root = Join-Path $work "atlas-cli-$Target"
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    "#!/bin/sh`nprintf 'atlas 9.9.9\n'`n" | Set-Content -Path (Join-Path $root "atlas") -NoNewline
    Copy-Item (Join-Path $RepoRoot "LICENSE") (Join-Path $root "LICENSE")
    Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $root "README.md")
    Copy-Item (Join-Path $RepoRoot "THIRD-PARTY-NOTICES.md") (Join-Path $root "THIRD-PARTY-NOTICES.md")
    tar -cJf (Join-Path $dist "atlas-cli-$Target.tar.xz") -C $work "atlas-cli-$Target"
  }

  function New-WindowsArchive($Target) {
    $root = Join-Path $tmp "atlas-cli-$Target"
    New-Item -ItemType Directory -Force -Path $root | Out-Null
    $appDir = Join-Path $tmp "stub-app"
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
Console.WriteLine("atlas 9.9.9");
'@ | Set-Content -Path (Join-Path $appDir "Program.cs")
      dotnet publish $appDir --configuration Release --runtime win-x64 --self-contained false --output (Join-Path $appDir "out") | Out-Null
    }
    Copy-Item (Join-Path $appDir "out/atlas-stub.exe") (Join-Path $root "atlas.exe")
    Copy-Item (Join-Path $RepoRoot "LICENSE") (Join-Path $root "LICENSE")
    Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $root "README.md")
    Copy-Item (Join-Path $RepoRoot "THIRD-PARTY-NOTICES.md") (Join-Path $root "THIRD-PARTY-NOTICES.md")
    Compress-Archive -Path $root -DestinationPath (Join-Path $dist "atlas-cli-$Target.zip")
  }

  @(
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu"
  ) | ForEach-Object { New-UnixArchive $_ }

  @(
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc"
  ) | ForEach-Object { New-WindowsArchive $_ }

  Copy-Item (Join-Path $RepoRoot "scripts/install/atlas-installer.sh") (Join-Path $dist "atlas-installer.sh")
  Copy-Item (Join-Path $RepoRoot "scripts/install/atlas-installer.ps1") (Join-Path $dist "atlas-installer.ps1")
  Copy-Item (Join-Path $RepoRoot "THIRD-PARTY-NOTICES.md") (Join-Path $dist "THIRD-PARTY-NOTICES.md")
  "{}" | Set-Content -Path (Join-Path $dist "dist-manifest.json")

  python (Join-Path $RepoRoot "scripts/release/generate-release-manifest.py") v9.9.9 $dist
  bash (Join-Path $RepoRoot "scripts/release/validate-release-assets.sh") v9.9.9 $dist

  $badManifestDist = Join-Path $tmp "bad-manifest-dist"
  Copy-Item $dist $badManifestDist -Recurse
  $manifestPath = Join-Path $badManifestDist "atlas-release-manifest.json"
  $manifest = Get-Content $manifestPath -Raw | ConvertFrom-Json
  $manifest.assets = @($manifest.assets | Where-Object { $_.target -ne "aarch64-apple-darwin" })
  $manifest | ConvertTo-Json -Depth 5 | Set-Content $manifestPath
  $acceptedBadManifest = $true
  try {
    bash (Join-Path $RepoRoot "scripts/release/validate-release-assets.sh") v9.9.9 $badManifestDist | Out-Null
  } catch {
    $acceptedBadManifest = $false
  }
  if ($acceptedBadManifest) {
    throw "asset validation accepted a manifest missing a required target"
  }

  $fakeBin = Join-Path $tmp "bin"
  $installDir = Join-Path $tmp "install"
  New-Item -ItemType Directory -Force -Path $fakeBin, $installDir | Out-Null
  @'
param(
  [string]$Uri,
  [string]$OutFile
)
$name = Split-Path ([System.Uri]$Uri).AbsolutePath -Leaf
Copy-Item (Join-Path $env:ATLAS_FAKE_RELEASE_DIR $name) $OutFile
'@ | Set-Content -Path (Join-Path $fakeBin "Invoke-WebRequest.ps1")

  @'
function Invoke-WebRequest {
  & (Join-Path $env:ATLAS_FAKE_BIN "Invoke-WebRequest.ps1") @args
}
& $env:ATLAS_INSTALLER -InstallDir $env:ATLAS_TEST_INSTALL_DIR -Version v9.9.9 -Yes
'@ | Set-Content -Path (Join-Path $fakeBin "run-install.ps1")

  $env:ATLAS_FAKE_BIN = $fakeBin
  $env:ATLAS_FAKE_RELEASE_DIR = $dist
  $env:ATLAS_INSTALLER = Join-Path $RepoRoot "scripts/install/atlas-installer.ps1"
  $env:ATLAS_TEST_INSTALL_DIR = $installDir
  $dryRunDir = Join-Path $tmp "dry-run-install"
  & $env:ATLAS_INSTALLER -InstallDir $dryRunDir -Version v9.9.9 -DryRun | Out-Null
  if (Test-Path (Join-Path $dryRunDir "atlas.exe")) {
    throw "PowerShell installer dry-run wrote an atlas.exe binary"
  }

  pwsh -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null

  $installedVersion = & (Join-Path $installDir "atlas.exe") --version
  if ($installedVersion.Trim() -ne "atlas 9.9.9") {
    throw "unexpected installed version: $installedVersion"
  }

  $secondOutput = pwsh -NoProfile -File (Join-Path $fakeBin "run-install.ps1")
  if (($secondOutput -join "`n") -notmatch [regex]::Escape("not on PATH")) {
    throw "PowerShell installer did not print PATH guidance"
  }

  @'
function Invoke-WebRequest {
  & (Join-Path $env:ATLAS_FAKE_BIN "Invoke-WebRequest.ps1") @args
}
& $env:ATLAS_INSTALLER -InstallDir $env:ATLAS_TEST_INSTALL_DIR -Version v9.9.9
'@ | Set-Content -Path (Join-Path $fakeBin "run-install-no-yes.ps1")
  $failed = $false
  try {
    pwsh -NoProfile -File (Join-Path $fakeBin "run-install-no-yes.ps1") | Out-Null
  } catch {
    $failed = $true
  }
  if (-not $failed) {
    throw "PowerShell installer replaced existing binary without -Yes"
  }

  $rollbackDist = Join-Path $tmp "rollback-dist"
  Copy-Item $dist $rollbackDist -Recurse
  $archive = if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -eq "Arm64") {
    "atlas-cli-aarch64-pc-windows-msvc.zip"
  } else {
    "atlas-cli-x86_64-pc-windows-msvc.zip"
  }
  $badRoot = Join-Path $tmp "bad-atlas"
  New-Item -ItemType Directory -Force -Path $badRoot | Out-Null
  Copy-Item (Join-Path $RepoRoot "LICENSE") (Join-Path $badRoot "LICENSE")
  Copy-Item (Join-Path $RepoRoot "README.md") (Join-Path $badRoot "README.md")
  Copy-Item (Join-Path $RepoRoot "THIRD-PARTY-NOTICES.md") (Join-Path $badRoot "THIRD-PARTY-NOTICES.md")
  "not an executable" | Set-Content -Path (Join-Path $badRoot "atlas.exe")
  Compress-Archive -Path $badRoot -DestinationPath (Join-Path $rollbackDist $archive) -Force
  python (Join-Path $RepoRoot "scripts/release/generate-release-manifest.py") v9.9.9 $rollbackDist
  $env:ATLAS_FAKE_RELEASE_DIR = $rollbackDist
  $failed = $false
  try {
    pwsh -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  } catch {
    $failed = $true
  }
  if (-not $failed) {
    throw "PowerShell installer accepted a binary that failed version validation"
  }
  $installedVersion = & (Join-Path $installDir "atlas.exe") --version
  if ($installedVersion.Trim() -ne "atlas 9.9.9") {
    throw "PowerShell installer did not restore previous binary after validation failure"
  }

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
    pwsh -NoProfile -File (Join-Path $fakeBin "run-install.ps1") | Out-Null
  } catch {
    $failed = $true
  }
  if (-not $failed) {
    throw "installer accepted a checksum mismatch"
  }

  Write-Host "PowerShell installer smoke tests passed"
} finally {
  Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
}
