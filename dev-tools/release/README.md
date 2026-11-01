# Private release tooling

Maintainer tooling for notices, release manifests, checksums and archive inspection.
This private package and its dependencies are excluded from Atlas distributions.
Use Node 22 or later, Rust/Cargo for dependency notices, and `tar` with XZ support.
The shell and PowerShell installer smoke tests exercise their native environments.

TypeScript implementation lives in `src/`, npm command entry points in `src/cli/`,
and tests and archive fixture construction in `tests/`. Operational shell and
PowerShell checks remain in `scripts/release/`. The build mirrors the package
directories under ignored `dist/`.

From the repository root:

```sh
npm --prefix dev-tools/release ci --ignore-scripts
npm --prefix dev-tools/release run verify
scripts/release/validate-release-tooling.sh
scripts/release/test-release-tools.sh
scripts/release/test-prepare-release.sh
```

On Windows, launch PowerShell 7 from Git Bash so the fixture builder uses Git's
XZ-capable tar: `pwsh -NoProfile -File scripts/release/test-installer.ps1`.
Windows' bundled tar cannot create XZ archives. The smoke test also uses the .NET
SDK to build a fixture executable. PR CI runs the package on Linux, macOS and
Windows and exercises the corresponding installer smoke tests.

The TypeScript build emits JavaScript into ignored `dist/`. The maintainer npm
commands build before execution and resolve path arguments from the caller's
working directory:

```sh
npm --prefix dev-tools/release run notices -- --check
npm --prefix dev-tools/release run notices
npm --prefix dev-tools/release run manifest -- v0.1.0 target/distrib
npm --prefix dev-tools/release run prune -- dist-manifest.json target/distrib
npm --prefix dev-tools/release run validate -- v0.1.0 target/distrib
```

Release jobs install and build the private package once, then call its emitted
entry points directly. `prepare-release.sh` regenerates notices through npm.
Invalid arguments exit 2; malformed inputs and failed validation exit 1.

Manifest asset entries include `variant` (`full` or `cli-only`), with full entries
first for older installers. Both variants retain lexical key ordering, ASCII escapes and
LF formatting. Hashes stream file bytes. ZIP inspection uses `yauzl` to read
central-directory names without decompressing binaries; `yazl` constructs test
fixtures. XZ tar archives use the system `tar`. Archive inspection never extracts
release files. The validator checks both variants for all four targets, checksum metadata,
cargo-dist references and excluded runtime-data paths. Its tag argument retains
the existing command interface; tag/version agreement belongs to the workflow's
earlier validation step.

These commands prepare or inspect files. Release publication stays in the
existing release workflow and explicit `prepare-release.sh --publish` flow.

## Nightly builds

`src/nightly.ts` owns nightly version preparation, source selection and publication;
`src/cli/nightly.ts` exposes those operations to `.github/workflows/nightly.yml`.
The workflow reuses the existing CI and packaging jobs for one main SHA. It stays
inactive on the integration branch and publishes only after reaching main.

After `npm --prefix dev-tools/release run build`, a read-only plan can be inspected
with `node dev-tools/release/dist/src/cli/nightly.js plan --scheduled`. Preparation
modifies the build checkout's CLI version and lockfile only. Publication validates
the assembled assets, keeps dated prereleases immutable, and tests the downloaded
Linux installer before advancing the nightly channel. The package tests exercise
skip/retry behavior and failure boundaries without network or GitHub writes.

The packaging matrix runs `smoke-packaged-web` against each extracted executable.
It starts `atlas web` on an OS-assigned localhost port from an empty temporary
working directory, with isolated source, index and model paths. It checks HTTP
status, content types and exact HTML/entry JavaScript/CSS bytes against the UI
bundle built by that job, then terminates the server and removes its temporary
directory. It needs no game data or model downloads. This checks embedded asset
serving; browser interactions and search readiness remain separate acceptance checks.

## Build variants

Each platform job builds one variant. Cargo-dist uses `precise-builds = true`
to isolate the shipped package; a workspace build would include `atlas-web` even
when the CLI disables it. CLI-only jobs set cargo-dist's
`default-features = false`, skip frontend preparation and run `stage-cli-only` to
rename the archive/checksum and rewrite its actual cargo-dist metadata under
`atlas-cli-only-<target>` identities. Assembly merges this metadata before pruning
references to absent artifacts. Full archive names remain compatible with older
installers. Both archive sets carry the release's third-party notices.

`smoke-packaged-cli` checks version, variant, command availability, JSON dice
output and workspace-scoped bundled skill installation in a disposable directory.
Full jobs additionally run `smoke-packaged-web`. Nightly publication tests both
downloaded variants before advancing its channel pointer.
