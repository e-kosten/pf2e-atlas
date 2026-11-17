# Installing and updating Atlas

For the default installation, follow the [README quick start](../README.md#quick-start).
This guide covers platform support, alternate versions and agent integration.

## macOS desktop app

On Apple Silicon with macOS Tahoe 26 or later, download
`atlas-desktop-aarch64-apple-darwin.dmg` from the
[Atlas release you want to install](https://github.com/e-kosten/pf2e-atlas/releases).
For a nightly, visit the [Latest Nightly download page](https://github.com/e-kosten/pf2e-atlas/releases/tag/nightly/latest).
Check the image's SHA-256 against `SHA256SUMS` on the same release. Open the
image and drag **PF2e Atlas** into **Applications**.

The app is ad-hoc signed and unnotarized. If macOS blocks first launch, try
opening the app, then go to **System Settings → Privacy & Security → Open Anyway**
to approve that download. See [Apple's first-open instructions](https://support.apple.com/en-us/102445).

Open the app and choose **Set up Atlas** on a new workspace. For upgrades,
quit Atlas, replace the app in Applications, reopen it, and use **Setup and
Updates** if Atlas asks to prepare the workspace. Stable and Nightly open the primary workspace by default. Use Release channel
in Atlas Desktop to review a switch and select the workspace to open.

The app includes the matching CLI. Use **PF2e Atlas → Terminal and Agent
Access** to link it at `~/.local/bin/atlas`. If you already installed a
standalone command there, choose **Switch to bundled CLI** and confirm the
replacement in the macOS dialog. Run `command -v atlas` to check which
command your shell uses. The linked command works when the app is closed
and follows the app when you replace it in Applications at the same path.
To switch back, rerun the standalone installer.

## Desktop updates

When a newer build is available on your installed channel, **Update** appears
beside Setup. Open it to see the version and any setup notice. Choose
**Install update** when you are ready to restart Atlas. If the update requires
setup afterward, follow the instructions Atlas shows on restart. An interrupted
download can be retried in Atlas.

Updater-enabled builds can install subsequent updates in the app. Install the
first updater-enabled release the same way you installed Atlas originally.

### Switch between Stable and Nightly

Open **Release channel** in Atlas Desktop. Choose the workspace you want to open
after switching and check the other channel's compatibility. When the verified
build can open or upgrade that workspace, Atlas shows the version and any setup
requirements before installing.

Take the primary workspace along to continue a campaign between compatible
releases. To try Nightly in isolation, choose **Trial workspace**. Atlas starts
it from a copy of the primary workspace and preserves both copies when you
switch back. The linked terminal command and standalone CLI still use the primary
workspace. To access the trial explicitly, pass the trial path shown in Release
channel with `--index`.

If Stable cannot open the selected workspace, choose **Stay on this build and
wait for Stable** to pause Nightly offers, or **Keep receiving Nightly updates
while waiting**. Atlas saves the choice across restarts and asks before
installing a compatible Stable. Change or cancel the request in Release channel.

Guided switching requires release builds with signed workspace compatibility.
If an installer was interrupted, follow the workspace selection recovery shown
on startup. When a supported workspace needs an upgrade, Atlas uses its normal
[workspace backup and upgrade procedure](setup-and-maintenance.md#upgrade-an-older-prerelease-workspace).
Older apps installed manually may reopen the primary workspace.

## Windows desktop app

On Windows x64, download `atlas-desktop-x86_64-pc-windows-msvc-setup.exe` from the
[Atlas release you want to install](https://github.com/e-kosten/pf2e-atlas/releases).
For a nightly, visit the [Latest Nightly download page](https://github.com/e-kosten/pf2e-atlas/releases/tag/nightly/latest).
Check its SHA-256 against `SHA256SUMS` on the same release. The installer is
unsigned, so Windows may show a warning when you open it.

Run the installer, open **PF2e Atlas** from the Start menu, and choose **Set up
Atlas** for a new workspace. To update, close Atlas and run the installer from
the newer release. When Atlas reopens, use **Setup and Updates** if prompted.

## Standalone CLI

On macOS or Linux, download and run the official installer:

```bash
(
  set -eu
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.sh -o "$installer"
  test -s "$installer"
  sh "$installer"
)
```

On Windows, use PowerShell:

```powershell
irm https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.ps1 | iex
```

Follow the installer's PATH instructions, then run `atlas setup`.
Before the first official release, use the [nightly installer](#nightly-builds).

## Update Atlas

Rerun the [standalone installer](#standalone-cli) to update the CLI to
the latest stable release, then run `atlas setup`. Restart a running web session.
The installer updates the executable; setup updates reference data while retaining
your saved lists and encounters. Atlas does not update itself automatically.

## Install without the web UI

Fresh installations include the CLI, agent tooling and web UI. To install only
the CLI and agent tooling on macOS/Linux:

```bash
(
  set -eu
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.sh -o "$installer"
  test -s "$installer"
  sh "$installer" --without-web
)
```

On Windows:

```powershell
$installer = Join-Path $env:TEMP 'atlas-installer.ps1'
Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/latest/download/atlas-installer.ps1 -OutFile $installer
& $installer -WithoutWeb
```

Saving the script under `$env:TEMP` works even when PowerShell starts in a
protected directory such as `C:\Windows\System32`.

Continue with `atlas setup` and, for your agent, `atlas agent skills install`.
CLI-only Atlas supports the existing CLI commands and semantic search. Its help
and completions omit `atlas web`.

Rerunning the installer preserves the build variant already installed at its
destination. Use `--with-web` (`-WithWeb` on Windows) to add the UI, or
`--without-web` (`-WithoutWeb`) to remove it. These options replace the same
`atlas` executable, including when its version is unchanged. Switching variants
at the same version keeps your workspace, saved work and model cache. After a
version upgrade, run `atlas setup` as usual.

Check your installed variant:

```bash
atlas build-info
atlas build-info --json
```

These commands work before setup. JSON uses the standard envelope with
`data.version` and `data.variant` (`full` or `cli-only`). `atlas --version` keeps
its existing format.

The CLI installers pin the selected stable or nightly release before downloading
its files. If the package reports a different version, installation stops and
keeps the previous command. A directory, directory symlink, or junction at the
command path must be moved before installing there.

The variant options also work with `--version` / `-Version` for stable, nightly
and specific releases. Releases from before CLI-only support provide the full
build; requesting CLI-only from one reports that the asset is unavailable.

## Platforms and versions

The installer adds the `atlas` command to your computer. Run `atlas setup` to
prepare the reference content and search model. See [data locations](setup-and-maintenance.md#data-locations)
for where Atlas keeps your saved work and downloaded files.

Release binaries are provided for Linux x64, Linux ARM64, macOS Apple Silicon,
and Windows x64. macOS Intel and Windows ARM64 are not published as release
binaries yet.

Confirm the installed version:

```bash
atlas --version
```

To install a specific version, use that release's installer URL (replace `VERSION`
with a published release tag such as `release/v1.2.3` or `rc/v1.2.3-rc.1`):

```bash
(
  set -eu
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/e-kosten/pf2e-atlas/releases/download/VERSION/atlas-installer.sh -o "$installer"
  test -s "$installer"
  sh "$installer" --version VERSION
)
```

For a release candidate, use its specific release tag; the default installer
selects the latest stable release. The `ATLAS_VERSION` environment variable also
selects a version for the installer.

## Nightly builds

Nightlies let you try recent changes before the next stable release. New builds
are available after automated checks pass. For the desktop downloads, see
[macOS](#macos-desktop-app) or [Windows](#windows-desktop-app).

To install or update the CLI to the newest successful nightly on macOS/Linux,
run the nightly installer:

```bash
(
  set -eu
  installer=$(mktemp)
  trap 'rm -f "$installer"' EXIT
  curl --proto '=https' --tlsv1.2 -LsSf \
    https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/latest/atlas-installer.sh -o "$installer"
  test -s "$installer"
  sh "$installer" --version nightly
)
```

On Windows, download the CLI installer and select the nightly version:

```powershell
$installer = Join-Path $env:TEMP 'atlas-installer.ps1'
Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/latest/atlas-installer.ps1 -OutFile $installer
& $installer -Version nightly
```

The installer prints the dated nightly version it installs. Run `atlas --version`
to see your installed version. Repeat the nightly installation command to update,
or use a specific dated tag with `--version <tag>` (`-Version <tag>` on Windows)
to reinstall that build. Add `--yes` / `-Yes` when running without prompts.

To return to stable, run the installer with `--version latest` / `-Version latest`.
Nightly and stable versions use the same workspace. [Back it up](setup-and-maintenance.md#back-up-and-restore-your-workspace)
before trying a nightly or switching to an older version. Changing the installed
version does not restore earlier saved data. Run `atlas setup` after upgrading.

## Manual download

Download the archive for your operating system from the
[releases page](https://github.com/e-kosten/pf2e-atlas/releases), verify it against
`SHA256SUMS`, extract `atlas`, put it on `PATH`, then run `atlas setup`.

Full archives are named `atlas-cli-<target>.tar.xz` (Windows: `.zip`). CLI-only
archives are named `atlas-cli-only-<target>.tar.xz` (Windows: `.zip`). Both contain
the command named `atlas` or `atlas.exe`.

Early release binaries are unsigned. macOS or Windows may show an operating-system warning until signing and notarization are added in a later release.

## Agent skill

Install Atlas's bundled skill to give your AI agent instructions for searching
and reading PF2e content. The bundled instructions match your installed Atlas
version:

```bash
atlas agent skills install
```

Choose the agent you use and whether to install the skill for one project or all
projects. For scripts, set these choices directly:

```bash
atlas agent skills install --target agents --scope workspace --yes --json
```

Supported targets are `agents`, `claude`, `codex`, `copilot`, `gemini`, and `kiro`.

To check installed copies of the skill, run:

```bash
atlas agent skills doctor
```

After upgrading Atlas, rerun the skill installer to update the installed copy.
If the installer reports that an existing copy differs, review any customizations
before replacing it with `--force`. Use the Atlas version you want the agent to
run when installing or updating its skill.

You can also install the skill independently with Node.js and the
[skills CLI](https://github.com/vercel-labs/skills):

```bash
npx skills add e-kosten/pf2e-atlas --skill pf2e-atlas-cli
```

The interactive installer lets you select agents and installation scope. For a
global Codex installation:

```bash
npx skills add e-kosten/pf2e-atlas --skill pf2e-atlas-cli --agent codex --global
```

This method installs the instructions separately from Atlas and its reference
data. It follows the latest development version; use `atlas agent skills install`
for instructions matched to your installed release. If you installed through
`npx skills`, update that copy using the same tool.

## Install from source

For source installation, use the Rust toolchain pinned in
[`rust-toolchain.toml`](../rust-toolchain.toml) and Node.js 22 (the CI version).
Build the frontend once, then install the CLI (which embeds that frontend):

```bash
npm --prefix web/atlas-ui ci
npm --prefix web/atlas-ui run build
cargo install --path crates/atlas-cli --locked
```

For CLI-only source installation, skip the frontend steps and disable the default
web feature:

```bash
cargo install --path crates/atlas-cli --locked --no-default-features
```

The CLI-only source build requires the pinned Rust toolchain; it does not require
Node.js or built frontend assets. Both builds use the same runtime data and agent
skill package.

See [CONTRIBUTING](../CONTRIBUTING.md) for source development and diagnostic commands.
