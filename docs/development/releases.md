# Release process

User-facing binaries are published from GitHub Releases. Maintainers need the GitHub CLI and cargo-dist:

```bash
gh auth login
gh auth status
cargo install cargo-dist --locked
```

Release tooling also requires Node 22 or later and an XZ-capable `tar`. Install the
private maintainer package with `npm --prefix dev-tools/release ci --ignore-scripts`
and validate it with `npm --prefix dev-tools/release run verify`. See
[release tooling](../../dev-tools/release/README.md) for notices, manifest, checksum and
platform smoke-test commands. These dependencies stay out of product bundles.

Each supported target ships a full archive (`atlas-cli-<target>.*`) and a CLI-only
archive (`atlas-cli-only-<target>.*`). Both contain `atlas`; full is the default.
Cargo-dist uses `precise-builds = true` to build the shipped CLI package rather
than every workspace member. The packaging matrix disables default features for
CLI-only without building the frontend, then stages cargo-dist's archive and metadata under distinct identities.
Assembly merges both sets of cargo-dist metadata. The Atlas release manifest
requires one asset per target/variant, with full entries first for older installers.
Keep that ordering, archive naming and the installer's legacy-manifest behavior
when changing packaging. See [ADR0064](../architecture/decisions/0064-optional-web-distribution.md).

### Nightly publication

`.github/workflows/nightly.yml` runs daily at 09:23 UTC and can be dispatched
manually from `main`. Scheduled runs skip the commit already served by the nightly
channel; manual runs rebuild it. A single source SHA is used for CI, packaging and
publication. Runs from other branches do not publish.

The workflow calls the existing CI and release-build-check workflows. Nightly
packaging edits only the CI checkout's CLI manifest and lockfile, using a version
such as `0.1.0-nightly.20261018092300.gabcdef123456`. No version-bump commits or
handwritten release notes are needed. The four-platform packaging checks run both variants before upload. The publication job validates all assets, publishes
a dated prerelease, and smoke-tests its downloaded Linux installer and binary.
Only then does it update `nightly-version.txt` in the `nightly` installer-channel
release. Installers resolve that pointer once and download all remaining assets
from the dated release. Dated releases remain available for rollback; the channel
release's tag is only an installer entry point, not the source identity of a build.

Nightlies are always prereleases and never become GitHub's latest stable release.
The stable workflow excludes nightly tags. Nightly writes use the job-scoped
`GITHUB_TOKEN` with `contents: write`; repository rules must permit this workflow
to create nightly tags and releases. No personal token is needed. If publication
fails, inspect the failed run and rerun it; published dated assets are not
replaced, and failed validation/install checks do not advance the channel.

After the integration branch merges into `main`, run `gh workflow run nightly.yml --ref main`,
inspect the complete hosted run, then install the nightly on the dogfooding machine
and check `atlas --version` and `atlas --help`. A local tooling test is not evidence
that the hosted platform builds or live installation succeeded.

Local release-tooling checks:

```bash
scripts/release/validate-release-tooling.sh
scripts/release/test-release-tools.sh
scripts/release/test-prepare-release.sh
npm --prefix dev-tools/release run verify
```

The Windows packaging job also runs `scripts/release/test-installer.ps1`. Fixture
tests cover pointer resolution and pinning, invalid pointers, checksum failures,
version preparation, scheduled skipping, and preserving the channel on failures.

### Stable releases and release candidates

Releases use a three-step flow:

1. From a clean, current `main`, create the release-preparation branch and scaffold the version bump and release notes:

```bash
scripts/prepare-release.sh --prepare-pr
# or: just release-prepare
```

The helper prompts for the release version, creates `release/v<version>`, updates
`crates/atlas-cli/Cargo.toml`, refreshes `Cargo.lock` and
`THIRD-PARTY-NOTICES.md`, and scaffolds `docs/releases/v<version>.md`. Pass
`--version <X.Y.Z[-rc.N]>` to skip the prompt. In an interactive terminal, the
helper opens the release notes with `VISUAL`, `EDITOR`, or `git config
core.editor` after creating the file. Edit the release notes and complete independent review of the release-preparation
changes before using the helper to validate, commit, and open the PR to `main`.

2. After editing the release notes and resolving independent review findings,
   have the helper validate, commit, push, and
   open the release-preparation PR:

```bash
scripts/prepare-release.sh --open-pr
# or: just release-open-pr
```

Independent review is a pre-PR requirement; this helper does not perform it.
The helper verifies the release-prep branch, checks that the release notes no
longer contain template text, runs local validation, commits the release-prep
files, pushes the branch, and opens a PR to `main`.

3. After the PR lands, publish the release from any clean checkout:

```bash
scripts/prepare-release.sh --publish --dry-run
scripts/prepare-release.sh --publish
# or: just release-publish-dry
# or: just release-publish
```

The helper switches to `main`, fast-forwards to `origin/main`, reads the
committed crate version, validates the matching release notes, and then creates
the tag and draft release. Pass `--version <X.Y.Z[-rc.N]>` only when you want an
extra guard that the committed crate version matches that exact value.

Running `scripts/prepare-release.sh` without a mode flag opens an interactive
picker for the workflow steps. Non-interactive shells must pass
`--prepare-pr`, `--open-pr`, or `--publish`. Interactive pickers use `fzf` when it is
available, with numbered prompts as the portable fallback.
Run `just release` to use the same interactive picker through the task runner.

Release candidates use Cargo prerelease versions and tags such as `0.1.0-rc.1` and `v0.1.0-rc.1`. The final release gets a separate version commit and rebuilds final artifacts as `0.1.0`; do not rename or promote RC artifacts.

Release notes use this template:

```md
# vX.Y.Z

## Summary

## Install Notes

## Known Issues
```

The helper creates an annotated tag and a draft GitHub release. The tag-triggered release workflow uses cargo-dist to build platform archives, then uploads Atlas installer scripts, checksums, cargo-dist metadata, the Atlas release manifest, and third-party notices. The draft is published only after the required asset set validates. If the workflow fails, the release remains draft.

cargo-dist is configured in `dist-workspace.toml`. The Atlas release workflow intentionally keeps repo-owned installers instead of cargo-dist-generated installers so install/update prompts, install locations, PATH guidance, and runtime-data policy remain under Atlas control.

To iterate on platform packaging without publishing a release, push a branch named
`release-build-check/<topic>` or `release-build-check-<topic>`. The
`release build check` workflow runs the cargo-dist build matrix, assembles the
same installer, manifest, checksum, and notices assets, validates the asset set,
and uploads the assembled output as workflow artifacts. It does not create a tag,
draft release, or GitHub Release asset. After the workflow exists on `main`, it
can also be run manually with an optional `vX.Y.Z[-rc.N]` tag override.

Repository settings checklist:

- GitHub Actions is enabled.
- Tag pushes trigger workflows.
- The release workflow can use `GITHUB_TOKEN` with `contents: write` for release asset upload.
- Only maintainers can push `v*` release tags.
- Branch protection allows release-preparation PRs to land normally.

Published release assets are treated as immutable. Fix bad published releases with a new patch release and document known issues in the affected release notes. Draft releases may be corrected before publication.

Local validation covers Rust checks, `dist plan`, release-helper dry runs, installer dry runs, release-tool smoke tests, and static script checks. GitHub-hosted CI owns platform matrix validation for Linux x64, Linux ARM64, macOS Apple Silicon, and Windows x64 release targets. macOS Intel and Windows ARM64 release binaries are deferred until the native ONNX Runtime packaging strategy supports them cleanly.
