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

The shared macOS packaging job links the static Apple compiler runtime resolved
by `xcrun --sdk macosx clang -print-libgcc-file-name`. ONNX Runtime's CoreML
availability checks require this helper when Rust links for the release deployment
target with its default libraries disabled. This adds the standard Apple runtime
helper without changing the deployment target or embedding policy.

The [Windows desktop build](../../desktop/README.md#windows-x64) produces
an unsigned per-user NSIS installer. The Windows desktop workflow uploads a
test build as a CI artifact. Release packaging builds the published installer
from the full Windows CLI archive for that exact release.

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

Nightly first looks for successful push-to-main CI for the exact planned SHA,
including its completed `validation complete` check and successful lockfile
scan within the repository tooling job. It waits up to
30 minutes for a pending run, then falls back to fresh CI when no qualifying
success exists. Failed, cancelled or incomplete evidence cannot authorize
packaging. Manual dispatch accepts `force-verification` to request fresh CI.
GitHub API errors fail the planning job. Reading prior runs requires `actions: read`.

Stable releases, nightlies and nonpublishing checks share the
`release-build-check.yml` build-and-assembly workflow. Its caller may mark release
tools verified only after successful full CI for the same source. Nightly does
so after reused or fresh verification; standalone checks and stable releases
verify their tools themselves. Tool verification builds the TypeScript output,
so no preliminary duplicate build is needed. Nightly
packaging edits only the CI checkout's CLI manifest and lockfile, using a version
such as `0.1.0-nightly.20261018092300.gabcdef123456`. No version-bump commits or
handwritten release notes are needed. The four-platform packaging checks run both
variants before upload. The publication job validates all assets, publishes a
prerelease under `nightly/<UTC timestamp>.g<12-character commit>`, and smoke-tests
its downloaded Linux installer and binary. Only then does it update
`nightly-version.txt` in the `nightly/latest` installer-channel release. Installers resolve that pointer once and download all remaining assets
from the dated release. Dated releases remain available for rollback; the channel
release's tag is only an installer entry point, not the source identity of a build.

Nightlies are always prereleases and never become GitHub's latest stable release.
The release workflow accepts only `release/v…` and `rc/v…` tags. Nightly writes use the job-scoped
`GITHUB_TOKEN` with `contents: write`; repository rules must permit this workflow
to create nightly tags and releases. No personal token is needed. If publication
fails, inspect the failed run and rerun it; published dated assets are not
replaced, and failed validation/install checks do not advance the channel.

After the integration branch merges into `main`, run `gh workflow run nightly.yml --ref main`,
inspect the complete hosted run, then install the nightly on the dogfooding machine
and check `atlas --version` and `atlas --help`. A local tooling test is not evidence
that the hosted platform builds or live installation succeeded.

### Desktop update signing

Stable and nightly builds produce an updater manifest and a signed package for
each desktop platform. The macOS update archive contains the same app bundle
as the DMG. Windows uses the same NSIS executable for both paths. Release
candidates stay outside the update feeds. The stable feed uses the latest
official stable release, and the nightly feed uses the existing dated-release
pointer. Both feeds resolve to tagged release assets.

Before the first updater-enabled production build, generate a Tauri key pair
with the desktop package's CLI.

```sh
npm --prefix desktop ci
desktop/node_modules/.bin/tauri signer generate --write-keys /secure/location/atlas-updater.key
```

Store the public key text as the `ATLAS_UPDATER_PUBLIC_KEY` GitHub Actions
repository variable. Store the private key text and its password in
`ATLAS_UPDATER_PRIVATE_KEY` and `ATLAS_UPDATER_PASSWORD` Actions secrets. Keep
a secure offline backup of the private key and password. Installed builds
trust this key, so losing it requires distributing a new updater-enabled
installer through the normal installation process. Apple and Windows code
signing are separate.

Signing runs in dedicated packaging steps. Tauri binds each signature to the
full Atlas version, including a nightly timestamp and commit suffix.
Packaging verifies the signer key ID and assembles both platform entries
before publication. The existing release digest checks include the update
assets. Standalone release build checks generate disposable signing keys and
exercise packaging without access to production secrets.

For a native upgrade rehearsal, use disposable workspace and app installs on
both platforms. Generate one disposable Tauri key pair for the two builds.
Set `ATLAS_UPDATER_PUBLIC_KEY` to the public key text when packaging each
build, and pass the disposable private key to
`desktop/updater-package.mjs sign` through `TAURI_SIGNING_PRIVATE_KEY_PATH`
and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. For test builds only, set
`ATLAS_UPDATER_REHEARSAL_ROOT` to `https://127.0.0.1:5467/releases` and
`ATLAS_UPDATER_REHEARSAL_CA` to the PEM text of a disposable CA certificate.
Use that CA to sign a separate TLS server certificate with an IP subject alt
name for `127.0.0.1`. Start the local feed with the server key and certificate
and the assembled newer release.

```sh
node desktop/rehearse-updater.mjs 5467 TLS_KEY TLS_CERT RELEASE_REF ASSEMBLED_DIRECTORY
```

Use the dated release ref, such as `release/v0.2.3` or
`nightly/20261116092300.gabcdef123456`. The test app trusts only the
configured certificate at that loopback origin. The production packaging
jobs use the GitHub Releases feed.

Check same-channel offers, a same-core nightly, rejection of another
release’s signature, dismissal, retry, and restart with both compatible and
setup-requiring workspaces. Keep production keys and the installed workspace
outside rehearsals.

### Desktop packaging and rehearsal

Stable releases, release candidates, and nightlies require both the Apple Silicon
DMG and the Windows x64 desktop installer. The full builds package and smoke-test
each desktop app with its matching CLI archive. Assembly validates the desktop
receipts against the archives and installer bytes. Both publication paths verify
the uploaded desktop digests before publishing. Nightly checks them again before
advancing the channel. Keep the original assembled artifact when retrying a
partially published release.

For an unpublished rehearsal, dispatch **release build check** on the
candidate ref. Use the checked-in CLI version for the official-style build.
For a nightly rehearsal, enable the `nightly` input and supply a package tag
with the candidate’s version core, UTC timestamp, and 12-character commit suffix,
such as `v0.1.0-nightly.20261018092300.gabcdef123456`. Download the assembled
artifact and test both desktop builds from disposable accounts.
[macOS installation](../installation.md#macos-desktop-app) ·
[Windows installation](../installation.md#windows-desktop-app)

Local release-tooling checks:

```bash
scripts/release/validate-release-tooling.sh
scripts/release/test-release-tools.sh
scripts/release/test-prepare-release.sh
```

When tools have not already been verified, the full Windows packaging job also
runs `scripts/release/test-installer.ps1`; the CLI-only job retains its own
packaged-binary smoke test. Fixture
tests cover pointer resolution and pinning, invalid pointers, checksum failures,
version preparation, scheduled skipping, and preserving the channel on failures.

### Stable releases and release candidates

Releases use a three-step flow:

1. From a clean, current `main`, create the release-preparation branch and scaffold the version bump and release notes:

```bash
scripts/prepare-release.sh --prepare-pr
# or: just release-prepare
```

The helper prompts for the release version, creates `release-prep/v<version>`, updates
`crates/atlas-cli/Cargo.toml`, refreshes `Cargo.lock`, and scaffolds `docs/releases/v<version>.md`. Pass
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

Independent review is required before merge readiness; this helper does not
perform it.
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

Release candidates use Cargo versions such as `0.1.0-rc.1` and tags such as
`rc/v0.1.0-rc.1`. The final release gets a separate version commit, rebuilds
the artifacts as `0.1.0`, and uses `release/v0.1.0`. GitHub marks the stable
release as latest for the default installers.

Release notes use this template:

```md
# vX.Y.Z

## Summary

## Install Notes

## Known Issues
```

The helper creates an annotated tag and a draft GitHub release. The tag-triggered release workflow uses cargo-dist to build platform archives, then uploads Atlas installer scripts, checksums, cargo-dist metadata, the Atlas release manifest, and third-party notices. The publisher compares every uploaded asset’s name and SHA-256 digest with the assembled files before publishing the same draft. If verification fails, the release stays in draft. Rerun with the original assembly to repair a partial upload.

If a tag was pushed and draft creation failed, use the tag and commit printed
by the helper:

```sh
scripts/prepare-release.sh --recover-draft TAG COMMIT --dry-run
scripts/prepare-release.sh --recover-draft TAG COMMIT
```

Recovery reads the version and release notes from that commit, so it also works
after `main` advances. Fetch the named tag from origin if the commit is missing
locally. If the tag-triggered workflow already failed waiting for the draft,
rerun its failed jobs after the draft exists.

cargo-dist is configured in `dist-workspace.toml`. The Atlas release workflow intentionally keeps repo-owned installers instead of cargo-dist-generated installers so install/update prompts, install locations, PATH guidance, and runtime-data policy remain under Atlas control.

The shared build check downloads the pinned cargo-dist archives in its prepare
job, verifies SHA-256, and shares one archive per platform with the packaging
jobs. The Actions cache can supply verified archives across runs. On a cache
miss, prepare retries transient upstream failures before failing the build.
When changing `cargo-dist-version` in `dist-workspace.toml`, update the version
and all four upstream SHA-256 pins in
`scripts/release/cargo-dist-checksums.json` together.

For acquisition failures, inspect the prepare job for the affected target,
cache status, and download or checksum error. If a failed-job rerun can no
longer download the tool artifacts from prepare, start a fresh workflow run.

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
- Reserve `release/v*` and `rc/v*` for maintainer-published releases.
- The main ruleset requires `validation complete`; this includes every selected
  gate and dependency advisories, with no separate path-filtered required workflow.
- Branch protection allows release-preparation PRs to land normally.

Published release assets are treated as immutable. Fix bad published releases with a new patch release and document known issues in the affected release notes. Draft releases may be corrected before publication.

Local validation covers Rust checks, `dist plan`, release-helper dry runs, installer dry runs, release-tool smoke tests, and static script checks. GitHub-hosted CI owns platform matrix validation for Linux x64, Linux ARM64, macOS Apple Silicon, and Windows x64 release targets. macOS Intel and Windows ARM64 release binaries are deferred until the native ONNX Runtime packaging strategy supports them cleanly.

### Refresh precomputed corpus embeddings

Official releases, including release candidates, require clean managed English
setup with both public seed sets and zero corpus inference on the packaged Linux x64
CLI. The same check runs locally with
`python3 scripts/release/check-embedding-seeds.py /path/to/atlas`.

Full-corpus coverage runs during release packaging. Ordinary PR validation can
pass while a staged migration still needs refreshed assets. To check a change
early, run the nonpublishing **release build check** workflow on its branch or
run the local checker with `--advisory` to report known gaps. Enable
`strict-embedding-coverage` when dispatching the workflow with nightly rehearsal
off to certify public pins on the packaged Linux x64 CLI.
A shared nightly can publish a known gap if other checks pass. Its release notes
name the affected profiles and show **Build embeddings locally** and
`atlas setup --build-embeddings-locally`. That explicit choice can take an hour
or more. Users can retry a transient download or wait for a nightly with updated
pins. An unexpected mismatch, failed download or bad checksum still blocks.
A numerical transition needs reviewed evidence in
`scripts/validation/embedding-seed-transitions.json`. Use the unreviewed-mismatch
diagnostic for the two numerical identities and the candidate manifest for the
exact pinned asset. The map is keyed by profile and each entry has `pin`, `previous` and `expected` fields. Keep the
separate [compatibility fixture review](#publish-an-embedding-compatibility-fixture)
applicable and remove transition evidence when updating the corresponding pin.

Build and verify from the reviewed Git ref carrying the previous public pins.
For example, use `origin/main` when it carries the desired pins.

```sh
just embedding-seed-refresh --baseline-ref origin/main --output scratch/seed-refresh
```

The command builds the candidate from the current clean checkout and downloads
the previous base and cumulative delta for each model. It reuses compatible
vectors in a disposable workspace and computes only missing vectors. For each
affected model, it exports one replacement delta with every current input the
base lacks. A model that is already covered keeps its pins. When its base alone
covers the corpus, the proposal can drop the delta pin.

Use `--rebuild-profile "profile=reviewed reason"` to roll over a base for an
incompatible numerical contract or a large cumulative delta. The receipt records
the reason and prior pins. In a second fresh workspace the command authenticates
all declared files, verifies both profiles with zero inference, and compares
semantic search against the producer. Review
`result/receipt.json` and, if it exists, `result/proposed-pins.json`. A fully
covered refresh reports an empty `replacements` object, writes the receipt and
keeps the existing pins. A refresh with a new BGE input and full Gemma coverage
proposes only a BGE delta.
After a failed run, rerun in the same output directory to reuse verified
downloads. Completed outputs are kept for review.

Once authorized, publish each replacement as a seed-only prerelease. Deltas use
`embedding-seed/<profile>/delta/v<N>`. Base rollovers use
`embedding-seed/<profile>/v<N>`. Retain the old releases for previous app
versions and keep latest disabled. If a proposed
tag already exists, rerun with `--next-revision <profile>=<number>` and a new
output directory. Verify the actual uploaded bytes and anonymous URLs before
applying the proposed pin change to `crates/atlas-runtime/embedding-seeds.json`.
Apply only the replacements from the proposal and keep any independently updated
pins, since the proposal starts from the reviewed baseline ref.
Use the literal slash tag in the pin. The final packaged check uses both
embedded pin sets, and a missing vector blocks the next official release that needs
it.

### Publish an embedding compatibility fixture

CI uses the fixture-only release [`fixtures/embedding-compatibility/v2`](https://github.com/e-kosten/pf2e-atlas/releases/tag/fixtures/embedding-compatibility/v2).
Each release target has a small predecessor workspace with expected search
results and a separate set of exact prepared inputs, token counts, and stored
vectors. The numerical samples can be replayed independently of the workspace
schema. These fixtures qualify embedding compatibility. The product's published
corpus seeds have their own [refresh procedure](#refresh-precomputed-corpus-embeddings).

| What changed | Fixture action |
| --- | --- |
| Routine Atlas release, compatible dependency update, or passage layout change | Keep the existing pins when compatibility checks pass. The fixture represents the predecessor whose vectors must remain usable. |
| A schema cutoff makes the pinned workspace unreadable | Coordinate the cutoff with a replacement workspace from a supported packaged predecessor for each affected target. Verify that its numerical contract still matches the pinned samples, and keep each numerical entry exactly as pinned. |
| An approved change increments a model's execution compatibility ID | First qualify the transition against the old pins. Once a packaged producer with the new ID is available, publish new workspace and numerical evidence for that profile on the release targets. Keep the other profile's pins. Follow the [transition procedure](dependencies.md#review-an-embedding-dependency-update). |

For a replacement, use the next fixture-only tag, such as
`fixtures/embedding-compatibility/v3`, at the verified producer commit. Validate
the capture and replay on each affected target. Create a prerelease draft with
latest disabled, upload the verified payloads, publish, then download them again
and check their SHA-256 digests before updating CI. Keep the previous release
assets so old pins and transition checks can still be reproduced.

Update `scripts/validation/embedding-fixtures-v2.json` in the activation PR.
It maps each target and profile to independent `workspace` and `numerical`
entries, each with the release `tag`, `asset`, `sha256`, and producer `commit`.
Verify CI against the exact downloaded assets before merging the new pins.

Run the `ci.yml` workflow manually to prepare unpublished fixture artifacts on
all four release targets. For a schema cutoff, pick a published CLI release
whose workspace schema the candidate supports and whose numerical contracts
match the pinned samples. Start a checked-in producer receipt from
[`schema-42-predecessor.json`](../../scripts/validation/schema-42-predecessor.json).
Use the release tag, the reported binary version without the `atlas ` prefix,
and its Git commit, and copy the four CLI-only archive
names and SHA-256 digests from `atlas-release-manifest.json`, cross-checking
them against `SHA256SUMS`. Record the tree ID of
`crates/atlas-ingest/tests/fixtures/foundry-source/core-record-types` at that
commit. Record the schema and ledger digest of a disposable index built by the
packaged CLI. From the repository root, use the probe's existing readers to get
those values. Keep the existing query recipe.

```bash
git rev-parse "$producer_commit:crates/atlas-ingest/tests/fixtures/foundry-source/core-record-types"
python3 - "$disposable_index" <<'PY'
from pathlib import Path
import runpy
import sys

probe = runpy.run_path("scripts/validation/probe-embedding-upgrade.py")
index = Path(sys.argv[1]).resolve()
print("schema", probe["workspace_schema"](index))
print("ledger_sha256", probe["predecessor_ledger"](index))
PY
```

The probe verifies the receipt against the actual archive, fixture tree, and
database on every target.

For a workspace-only refresh, dispatch from the branch containing that receipt.

```bash
gh workflow run ci.yml --ref "$capture_branch" \
  -f capture-embedding-fixtures=true -f capture-workspace-only=true \
  -f fixture-producer-receipt=scripts/validation/new-producer.json
```

Each `embedding-v2-fixtures-*` workflow artifact includes the workspace,
its digest, and `fixture-replay.json` with the producer and candidate identities,
schema versions, and numerical receipt checked for each profile. This dispatch
replays the replacement workspace even if the candidate can no longer open the
published workspace. Use the publication and activation steps above once the
replacement is ready.

To reproduce the original full capture, dispatch with only
`-f capture-embedding-fixtures=true`. The default schema-42 receipt produces
both workspace and numerical payloads. A later full capture can take a producer
receipt with the same execution IDs, provided the numerical input helper works
with that producer's source. Use the separate transition procedure when a
profile's execution ID changes.

Stable and RC packaging runs the compact compatibility probe against the exact
packaged binaries when changes since the pinned producer affect numerical
execution. The full manual probe still reads its workspace from the legacy
producer pin. When a cutoff or execution ID change invalidates that pin, update
the manual probe to use a compatible legacy fixture or move it to the split
fixtures. Changing only the policy ID cannot qualify the old archive.

#### Original single-archive fixtures

For a checkout that still uses the original format, generate the fixture with
the packaged Linux x86_64 CLI from a pinned release build check on main or a
verified product release. Record the actual target and build when generating
on another platform.

```sh
python3 scripts/validation/probe-embedding-upgrade.py \
  --baseline /absolute/path/to/packaged/atlas \
  --producer-commit <commit> --producer-target x86_64-unknown-linux-gnu \
  --producer-build <packaging-run-url-or-product-tag> \
  --model-cache "$ATLAS_TEST_MODEL_CACHE" --output scratch/baseline-capture \
  --produce scratch/embedding-compatibility-baseline.tar.gz
```

Pass `--profile bge-base` or `--profile embeddinggemma` when replacing a single
profile's fixture. Inspect the archive contents and run the probe with
`--bundle`, its reported `--sha256`, and a fresh output directory before
publication. After publication is authorized, create a fixture-only draft under
a new `fixtures/embedding-compatibility/v<N>` tag at the producer commit.
Mark the new release as a prerelease with latest disabled. Attach the verified
archive to the draft and publish it. Download the published bytes and rerun the probe before
pinning.

In the CI activation PR, update the affected profile entry in
`scripts/validation/embedding-baseline.json`. A bundle entry records `tag`,
`asset`, `sha256` and `execution_policy` from the published bundle. Preserve
earlier assets so the previous pin can still be reproduced.
If an asset becomes unavailable, restore the exact archive at a new fixture tag
and update the pin with the same SHA-256 and execution policy. If the exact bytes
are unavailable, review fixture recovery separately before changing the pin.
See [dependency maintenance](dependencies.md#review-an-embedding-dependency-update)
for the review procedure.
