# Installing and updating Atlas

For the default installation, follow the [README quick start](../README.md#quick-start).
This guide covers platform support, alternate versions and agent integration.

## Update Atlas

Rerun the [quick-start installer](../README.md#quick-start) to update the CLI to
the latest stable release, then run `atlas setup`. Restart a running web session.
The installer updates the executable; setup updates reference data while retaining
your saved lists and encounters. Atlas does not update itself automatically.

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
with a published release tag):

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/download/VERSION/atlas-installer.sh | sh
```

For a release candidate, use its specific release tag; the default installer
selects the latest stable release. The `ATLAS_VERSION` environment variable also
selects a version for the installer.

## Nightly builds

Nightlies let you try recent changes before the next stable release. New builds
are available after automated checks pass. The default quick-start installer uses
the stable release; use the commands below to choose a nightly instead.

Install or update to the newest successful nightly on macOS/Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/atlas-installer.sh \
  | sh -s -- --version nightly
```

On Windows, download the installer and select the nightly version:

```powershell
Invoke-WebRequest https://github.com/e-kosten/pf2e-atlas/releases/download/nightly/atlas-installer.ps1 -OutFile atlas-installer.ps1
./atlas-installer.ps1 -Version nightly
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

See [CONTRIBUTING](../CONTRIBUTING.md) for source development and diagnostic commands.
