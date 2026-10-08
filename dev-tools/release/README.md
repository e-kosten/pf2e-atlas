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

Manifests retain their existing schema, lexical key ordering, ASCII escapes and
LF formatting. Hashes stream file bytes. ZIP inspection uses `yauzl` to read
central-directory names without decompressing binaries; `yazl` constructs test
fixtures. XZ tar archives use the system `tar`. Archive inspection never extracts
release files. The validator checks the four target packages, checksum metadata,
cargo-dist references and excluded runtime-data paths. Its tag argument retains
the existing command interface; tag/version agreement belongs to the workflow's
earlier validation step.

These commands prepare or inspect files. Release publication stays in the
existing release workflow and explicit `prepare-release.sh --publish` flow.
