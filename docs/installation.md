# Installing and updating Atlas

For the default installation, follow the [README quick start](../README.md#quick-start).
This guide covers platform support, alternate versions and agent integration.

## Update Atlas

Rerun the [quick-start installer](../README.md#quick-start) to update the CLI to
the latest stable release, then run `atlas setup`. Restart a running web session.
The installer updates the executable; setup updates reference data while retaining
your saved lists and encounters. Atlas does not update itself automatically.

## Platforms and versions

The installer installs only the `atlas` CLI. The persistent workspace lives in user application-data storage; downloaded source and models stay in cache locations managed by `atlas setup`.

Release binaries are provided for Linux x64, Linux ARM64, macOS Apple Silicon,
and Windows x64. macOS Intel and Windows ARM64 are not published as release
binaries yet.

Confirm the installed version:

```bash
atlas --version
```

To install a pinned version, use that release's installer URL (replace `VERSION`
with a published release tag):

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/e-kosten/pf2e-atlas/releases/download/VERSION/atlas-installer.sh | sh
```

Release candidates are installed only from pinned URLs or with `ATLAS_VERSION`.

## Nightly builds

Nightlies are prerelease builds of `main` for everyday testing. They are published
when the scheduled build finds a new commit and all checks pass. Stable releases
remain an explicit maintainer action; the default installer selects stable.

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

The installer prints the dated release it resolved. `atlas --version` includes
that build's UTC timestamp and source commit prefix. Each dated release retains
its binaries, so you can reinstall a specific nightly with `--version <tag>`
(or `-Version <tag>`). Repeat the nightly command to update; Atlas does not
update itself automatically. Add `--yes` / `-Yes` for unattended replacement.

To return to stable, run the installer with `--version latest` / `-Version latest`.
Nightly and stable installations share the normal data paths; switching binaries
does not restore older local data. Back up saved state before testing changes that
migrate it, and rerun `atlas setup` if the selected build requires a rebuilt index.

## Manual download

Download the cargo-dist archive for your OS from the
[releases page](https://github.com/e-kosten/pf2e-atlas/releases), verify it against
`SHA256SUMS`, extract `atlas`, put it on `PATH`, then run `atlas setup`.

Early release binaries are unsigned. macOS or Windows may show an operating-system warning until signing and notarization are added in a later release.

## Agent skill

The CLI includes a first-party skill that teaches local coding agents how to use Atlas for PF2E lookup, search, filter discovery, and graph context.

With Atlas installed, use its bundled skill (matched to that CLI version) and
choose the agent target you use:

```bash
atlas agent skills install
```

For scripts or setup automation, provide the target and scope directly:

```bash
atlas agent skills install --target agents --scope workspace --yes --json
```

Supported targets are `agents`, `claude`, `codex`, `copilot`, `gemini`, and `kiro`.

To inspect existing installs or diagnose a changed skill directory, use:

```bash
atlas agent skills doctor
```

After upgrading Atlas, rerun the skill installer to update the installed copy.
A differing copy requires `--force`; review local edits before replacing it.
The skill is bundled with the executable, so install it from the Atlas version
you intend the agent to use.

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

This installs only the skill, not Atlas or its data. The skill includes CLI
installation and setup guidance. The repository's `main` skill tracks the nightly
CLI; use the bundled installer for a release-matched copy. Use the installation
method you chose to update that copy; Atlas's installer does not manage the
skills CLI's symlinks or lockfiles.

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
