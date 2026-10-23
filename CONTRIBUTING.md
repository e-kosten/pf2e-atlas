# Contributing

## Worktrees And Pull Requests

Implementation uses dedicated linked worktrees and feature PRs:

- create a task worktree under `.worktrees/<task-slug>` on a short-lived feature branch
- branch from current `main` unless the user identifies another workstream; dependent PRs may target the preceding feature branch
- continue related feedback and revisions in the same task worktree and PR
- preserve unrelated changes and running builds being reviewed
- land through a reviewed PR after applicable validation and CI; merging is a separate human decision

Recommended branch names:

- `feat/<topic>`
- `fix/<topic>`
- `docs/<topic>`
- `chore/<topic>`

For agents, authorization to implement includes logical local commits, pushing the task branch, and opening or updating its PR. Separate permission is not needed for these routine steps. Planning alone does not authorize implementation, and explicit plan-only, no-commit, or local-only instructions take precedence. Do not merge, push directly to shared integration branches or `main`, publish releases, or modify the user's real installed Atlas workspace without explicit authorization. Repository-local development data follows the standing cleanup authorization in [Development Data And Task Cleanup](#development-data-and-task-cleanup).

Use a draft PR while implementation or required validation remains incomplete. Describe the resulting behavior, validation evidence, and limitations. Keep review fixes visible as additional logical commits; agree on any published-history rewrite before performing it.

Keep the worktree available while its PR or local build is being reviewed. After merge or cancellation, stop its task-owned previews unless the user requested continued access, then remove the checkout as part of task completion. Preserve needed uncommitted source work and check for another active agent, build or dependency. At the next task's start, finish outstanding cleanup. Retaining a branch does not require retaining its checkout or compiler cache.

Before removal, inspect Git status, untracked and ignored files, running previews/builds, and incoming symlinks. Preserve needed uncommitted source files and explicitly retained data; delete disposable development state instead of archiving it. A live `node_modules` symlink target must remain even if its original PR has merged. Remove worktrees with `git worktree remove`, one at a time, and verify the inventory. Use `--force` only after accounting for every needed file it would discard; do not broadly delete `.worktrees/`.

For retained worktrees, record the path, purpose, owner, and removal condition. Keep only the resources needed by an active task or explicitly retained preview; a stable preview normally does not require a compiler cache. Temporary review checkouts last only as long as their review.

## Development Loop

Complete the agreed scope autonomously, including necessary integration, tests,
documentation and review fixes. Logical commits organize finished parts of the
work; they do not require tiny PRs, reduced scope or repeated user approvals.
Ask for input when a consequential product/architecture decision is unresolved,
a real blocker needs the user, or the early UX checkpoint below is due.

During iteration, run focused tests for the behavior being changed. Before marking
the PR ready, run the full applicable gates and review the resulting diff against
the agreed outcome. Scale additional independent reviews to risk and explicit
requirements; they are not a mandatory ceremony for every small change. Preserve
the required browser/Astra review for UI changes.

Reuse successful checks for unchanged code, inputs and toolchain. Record the
candidate and commands; rerun affected checks after substantive changes. An
unrelated documentation edit need not invalidate a completed compiler/test run.
Local validation and CI both matter: local checks shorten feedback, while CI
checks the published candidate in clean environments and on other platforms.

## Local Build Review

User experience changes, including CLI behavior and output, get an early human
checkpoint once the core flow is functional. Provide a runnable preview or sample
CLI commands and output after minimal usability checks, then pause for feedback
before broad test rewrites and final validation. Clearly identify unfinished
behavior and validation gaps. Existing tests remain intact, and all applicable
final gates and applicable independent reviews still follow once the UX direction is agreed.

Provide a stable local build when a change is useful to inspect. State its branch, commit, URL or run command, and any sample-data limitations. Disclose uncommitted differences if present. Preserve that build while developing further changes, using a separate checkout or built artifact when necessary, and explicitly identify when an updated build replaces it.

Human UI previews use a compatible full-corpus development workspace and shared
verified model cache, with every configured profile usable. Reference data and
saved state live in one SQLite file. Reuse an existing development backend for
frontend-only work when shared preview state is acceptable; otherwise seed a
private workspace with `just dev-workspace-init`. Do not attach a candidate to
the installed personal workspace. Use Vite against the selected backend for hot
reload. The model-free browser fixture is for automated acceptance and targeted
fault checks, not the human product preview. See the baseline workflow below.

UI changes require interaction and visual review alongside automated checks. Agents use Astra sub-agents for presentation review, including actual browser inspection. Local validation, hosted CI, and human acceptance are separate outcomes; a passing gate does not establish visual acceptance.

## Working Plans

Substantial tasks may use one concise local plan under `scratch/plans/`; ordinary
fixes do not require one. Refine the same plan as scope evolves, using an explicit
status and concrete acceptance checks. The local `scratch/plans/README.md`, when
present, indexes current and deferred work. Archive completed or superseded plans,
update mutable links, and retain provenance-bound originals. Plans are local working
artifacts; architecture docs and the tracked backlog remain durable shared guidance.
Use the task's appropriate checkout while planning. Plan approval and execution
authorization are separate; keep planning uncommitted unless explicitly requested.

## Commit Messages

Use Conventional Commits for all changes. Every commit message must include a Conventional Commit summary line. A short description body is optional, but when present it must appear after a blank line.

Commit at logical, validated boundaries rather than accumulating an entire multi-stage task uncommitted. Each commit should express a coherent change, include its relevant tests and documentation, and leave the branch buildable. A substantial PR may contain several commits. Avoid arbitrary progress snapshots and unrelated changes in the same commit.

Summary-only format:

```text
type(scope): summary
```

Summary-plus-body format:

```text
type(scope): summary

description
```

Recommended types:

- `feat` for new CLI, runtime, ingest, artifact, or search capabilities
- `fix` for bug fixes or behavior corrections
- `docs` for README, architecture, backlog, or contributor guide updates
- `refactor` for internal code restructuring without behavior changes
- `test` for test-only changes
- `chore` for maintenance, scripts, or repo setup

## Project Shape

PF2e Atlas is a Rust workspace:

- `Cargo.toml`: workspace definition and shared dependency versions
- `crates/atlas-cli`: product command parsing, presentation, exit codes, and agent skill installation
- `crates/atlas-dev`: private Rust developer source and artifact diagnostics
- `crates/atlas-cli-support`: shared argument vocabulary, JSON envelopes, and progress rendering
- `crates/atlas-runtime`: path resolution, setup readiness, source-fetch policy, and runtime handle construction
- `crates/atlas-search`: product-facing retrieval orchestration
- `crates/atlas-storage`: bounded connections, process admission, migrations and backups
- `crates/atlas-index`: reference schema, incremental publication, checkpoints, validation, row readers, filter discovery, filter compilation, and vector SQL
- `crates/atlas-foundry-model`: generated authored DTOs, field-retaining admission and checked snapshot codecs
- `crates/atlas-ingest`: typed Foundry loading, preparation, attributed indexing inputs, embedding execution and artifact-build composition
- `crates/atlas-embedding`: two pinned search-profile contracts, shared BGE segmentation, native final budgets, FastEmbed inference and retained shared query sessions
- `crates/atlas-record`: source-backed records, borrowed authored views, selected content preparation, query facts and reference policy
- `crates/atlas-domain`: shared typed query, source address, lightweight summary and canonical record-key vocabulary
- `crates/atlas-app-service`: saved-list and encounter workflows, local overlays and consumer presentation projections
- `crates/atlas-app-model`: generated frontend contracts for product workflows and presentation
- `crates/atlas-local-state`: saved lists, encounters and versioned receipts on caller-owned workspace sessions
- `crates/atlas-web`: web transport and embedded frontend hosting
- `crates/atlas-sqlite-vec`: sqlite-vec registration and capability probing
- `skills/pf2e-atlas-cli`: first-party local-agent skill installed by `atlas agent skills`
- `dev-tools/`: private TypeScript packages for source contracts and release tooling,
  with implementation, command entry points, tests and fixtures in separate directories
- `scripts/`: operational shell and PowerShell workflows, verification and installers

Architecture notes live under [`docs/architecture`](./docs/architecture/overview.md):

- [`overview.md`](./docs/architecture/overview.md): architecture landing page and crate navigation
- [`runtime.md`](./docs/architecture/runtime.md): crate ownership, ingest flow, projections, and runtime search architecture
- [`artifact-contract.md`](./docs/architecture/artifact-contract.md): SQLite artifact schema and validation contract
- [`decisions/`](./docs/architecture/decisions/README.md): architecture decision records

## Development

`README.md` is the user-facing product and setup document. Keep contributor workflow, internal command surfaces, and repo-shape guidance here instead of expanding the README with developer-oriented detail.

Install tracked git hooks and verify this checkout:

```bash
scripts/install-git-hooks.sh
scripts/preflight.sh
# or: just dev-setup
```

Use Cargo's default `target/` directory inside each checkout, including linked
worktrees. Compiled tests embed checkout-specific fixture paths, so sharing a
`CARGO_TARGET_DIR` across worktrees can reuse binaries with stale paths. Run
`git push` directly; no Cargo environment override is needed.

### Reusable Development Baseline

The existing Rust `atlas-dev` tool owns the baseline workflow. On macOS or Linux,
run `just dev-tool-build` once from a checkout containing this tooling. It uses the
normal Cargo build and retains the executable under the primary checkout at
`.atlas/dev-baselines/tools/atlas-dev`. Subsequent init/status/web commands need
no Rust build or Python runtime, even from a new worktree.

Refresh with `just dev-tool-build` after changing the tooling or when publication
reports a schema mismatch. Refreshing replaces only the retained developer tool;
it does not change baseline executables, manifests, data, or existing previews.
`ATLAS_DEV_BINARY=/absolute/path/to/atlas-dev` explicitly selects a candidate
developer tool for the thin shell launchers. The selected Atlas app executable
is a separate choice controlled by `--binary`.

All linked worktrees resolve one shared store through Git's common directory:
`<primary-checkout>/.atlas/dev-baselines`. `--store /absolute/path` overrides it.
The store contains versioned executable/database pairs and an atomic `current.json`
pointer. It is development data, not disposable compiler output.

Publish an existing build whose provenance you know, after its normal code checks.
Publishing performs a consistent SQLite backup, checks Atlas readiness, runs deep
artifact validation and executes a vector query with every configured profile.
It does not compile, fetch models, migrate, or ingest. The source workspace must
already be ready. **All saved state is copied**: use a dedicated development seed
with controlled examples, not your personal installed workspace.

```sh
just dev-baseline-publish --binary target/debug/atlas \
  --build-commit HEAD --build-profile dev --toolchain 1.95.0 \
  --index .atlas/workspace.sqlite \
  --source /absolute/shared/pf2e/PINNED_COMMIT \
  --embedding-cache-path /absolute/shared/hf-models
```

The commit, toolchain and profile are publisher-declared build provenance; the
command cannot infer them from an arbitrary binary. Use the actual build commit,
not current HEAD if the executable is older. The manifest also records the copied
binary hash/version, database hash, schema version, source/contract context,
embedding profiles and validation reports. Default asset resolution uses global
paths; explicit source and model paths must survive worktree retirement. Publication
rejects assets inside registered linked worktrees. Neither Cargo targets nor the
installed personal executable are overwritten.

For main or a new worktree:

```sh
just dev-workspace-init
just dev-status
just dev-web --port 4727
# Frontend-only work: point this checkout's Vite server at that backend.
ATLAS_API_PROXY=http://127.0.0.1:4727 npm --prefix web/atlas-ui run dev
```

Initialization copies the current seed into `.atlas/workspace.sqlite`, records
its baseline in `.atlas/dev-workspace.json`, and never replaces existing data or
receipts. On macOS it attempts an APFS clone, falling back to a regular copy.
Each database has independent mutable pages and stable Atlas lock files. An
existing database without a receipt is preserved, not adopted or reset.

`dev-status` and `dev-web` default to the frozen baseline executable. They do not
claim to run the current checkout's code. For Rust changes, build in the checkout's
own warm `target/`, then select that executable explicitly:

```sh
cargo build --locked -p atlas-cli
just dev-status --binary target/debug/atlas
just dev-web --binary target/debug/atlas --port 4827
```

`dev-status --json` includes the complete manifest and check reports.
Status invokes read-only Atlas checks and prints a fully qualified setup command
when a candidate needs migration or a reference update. It exits nonzero for
incompatible/unready data. Run that setup command deliberately against the private
copy; failed readiness never triggers an automatic rebuild. Setup can reuse
compatible saved embeddings. `--binary` may point to `target/debug-full/atlas` for
full debugger symbols or a release build for indexing/inference performance.

Each working copy stays pinned to its original seed even after a new default is
published. Initialization accepts `--baseline ID` to select an older version for
a new workspace. Existing lists/encounters must be preserved through normal
setup; there is intentionally no overwrite/reset flag. Do not point the web app
at the immutable seed itself, or hard-link writable SQLite databases. Seeds retain
the copied workspace identity; use separate preview ports and do not exchange
saved-state generations between divergent copies.

Publish new baseline versions when a validated main build or data-contract change
makes one useful; ordinary presentation edits need no data rebuild. Retain the
main Cargo target for incremental work. Parallel Rust branches still compile
locally; a baseline executable serves branches that do not need backend changes.
Keep baseline versions referenced by `.atlas/dev-workspace.json` or running
processes. Retire unused versions only after checking those dependencies and any
saved state. Existing previews never follow the moving default automatically.

The commands can be tried before merge from another checkout with
`just --justfile /absolute/path/to/task/justfile dev-status`; they operate on the
invoking checkout. `--checkout /absolute/path` selects a different checkout.
`just dev-baseline-test` runs the focused snapshot/publication/isolation tests
without models. The same tests run in the existing Rust workspace CI gate.

### Build Profiles And Disk Budget

Ordinary development and tests use limited debug information and enable
incremental compilation. This keeps source/function backtraces and qualified
function names and reuses compiler work during editing. The large generated
`atlas-foundry-model` crate disables incremental compilation to avoid retaining
its extra compiler cache; Cargo still reuses its unchanged completed output.
Clean CI jobs set `CARGO_INCREMENTAL=0` globally. Tests inherit development settings,
including debug assertions and overflow checks; release uses its own profile.

For a debugger session that needs variable/type inspection, use the full-debug
profile for both builds and tests:

```bash
cargo build -p atlas-cli --profile debug-full
cargo test -p atlas-local-state --profile debug-full
```

That profile writes to `target/debug-full`. Retain it while debugging; remove it
when the session is over and the output is no longer needed:

```bash
cargo clean --profile debug-full
```

Use the checked-in defaults consistently across manual checks and hooks. Avoid
profile/env changes for individual agents or validation retries, and retire
obsolete configurations rather than retaining both indefinitely. Normal
`git push` needs no override. Keep active targets: routinely deleting and
rebuilding them adds latency and disk writes. Neither retained size nor build
time alone establishes SSD write volume or endurance.

Before creating a worktree or running expensive gates, check free space and the
current task's output:

```bash
df -h .
git worktree list
test ! -d target || du -sh target
```

Below 30 GiB free, reclaim obsolete task build output before starting another
large build. A target above 15 GiB warrants inspecting `debug/deps`,
`debug/incremental`, and `debug/build` for accumulated configurations. These
thresholds prompt investigation, never automatic deletion of active files.
Monitor growth during long builds. Preserve shared source/model assets and
artifacts while an active task or review needs them.

Keep one checkout-local Cargo target per task; do not copy or symlink Cargo
outputs across checkout paths. Stop completed tasks' previews before retirement,
unless the user requested continued access. Check for other active builds,
reviews and dependency links before deleting output. Report what was removed,
what must remain and why, and measured space recovered in the final response.

### Development Data And Task Cleanup

All repository-local runtime and test state is disposable by default. This includes
preview databases, lists and encounters edited during review, migration backups,
fixture exports and copied reference corpora. Task completion supplies standing
permission to delete these resources without asking again. Preserve uncommitted
source work, another agent's active resources and any data the user explicitly
asks to retain.

The user's real installed Atlas workspace is protected. Resolve symlinks and
configured paths before deleting data: a path under the repo can point to the real
installation, and a command using global path mode can select a development
workspace through an explicit override. Never reset, replace or delete the real
installed workspace without explicit authorization. Normal Atlas setup continues
to preserve saved state; the development cleanup policy does not change product
behavior.

Reuse one workspace per active task. Keep a separate frozen preview only when
review requires it, and remove superseded copies promptly. Preserve the shared
baseline while active checkouts reference it. A completed task's disposable
database does not need to survive merely because it contains saved rows.

Do not routinely archive databases, backup copies, executables, Git bundles or
entire retired worktrees under `scratch/`. Do not create export scripts or recovery
archives to preserve disposable fixtures. Commit/PR references and concise
validation results normally suffice; keep larger artifacts only for an unresolved
failure, active review or explicit user request, with a concrete removal milestone.
Moving obsolete files into a cold archive is not cleanup. No archive README or
persistent cleanup manifest is needed solely to record deletion.

The Rust web server embeds the built frontend in both debug and release binaries.
Build the frontend before compiling a preview binary, then keep that binary and
its reference artifact fixed while they are under review. Later frontend builds
do not update a running Rust server; restart it with the new binary intentionally.
Use the Vite dev server when frontend hot reload is wanted.

Build and test from the repository root:

```bash
just verify
# or: scripts/verify.sh
```

Validation is quiet by default: successful gates print summary lines, and detailed
Cargo output is replayed only when a gate fails. Use `just verify --verbose` or
`scripts/verify.sh --verbose` when you want the full command stream.

Run the CLI from source:

```bash
cargo run -p atlas-cli -- --help
cargo run -p atlas-cli -- setup --check --json
cargo run -p atlas-cli -- search "low level healing spell" --kind spell --limit 5
```

Install the CLI from this clone:

```bash
cargo install --path crates/atlas-cli --locked
# or: just install
```

Generate shell completions:

```bash
atlas completions zsh
atlas completions bash
atlas completions fish
```

## Rust Developer Commands

Build and run developer diagnostics independently of the product CLI, web UI
bundle, and Node:

```bash
cargo run -p atlas-dev -- --help
cargo run -p atlas-dev -- source load --source vendor/pf2e --json
cargo run -p atlas-dev -- source analyze --source vendor/pf2e --json
cargo run -p atlas-dev -- source audit-paths --source vendor/pf2e --record-type npc --json
cargo run -p atlas-dev -- index inspect --index .atlas/workspace.sqlite --json
cargo run -p atlas-dev -- index record actionspf2e:1kGNdIIhuglAjIp9 --index .atlas/workspace.sqlite --json
cargo test -p atlas-dev -p atlas-cli-support
```

These commands use the same global/repo path policy, JSON envelope, and progress
controls as atlas. `source load` reads generated authored DTOs, keeping original
bytes, raw values, provenance and every admission
diagnostic. Its report counts modeled, partial, raw-only and quarantined outcomes
separately. It applies no source defaults, coercions, metric extraction or
embedding work. Embedded children and authored links remain in their source DTOs;
generic rule retention does not imply specific-rule interpretation.
Partial modeled documents produce exit 0; raw-only roots, quarantine, unavailable
packs or empty input produce exit 1 with the report on stdout. Root/manifest
errors produce exit 2. JSON `status: ok` means a report was produced; check the
counts and exit code for completeness. The loading API retains the full source
corpus in memory, with quarantine bytes retained only for successful file reads.
Analysis reports typed source selection and admission without writing SQLite;
path auditing reports observed paths and values. Index inspection reads an
existing artifact without changing it. `index record` decodes its checked source
snapshot and developer provenance; `--original --source vendor/pf2e` explicitly
reads the original clone file only after checking its recorded path and hash.
Source commands' `--manifest` overrides
the Foundry input manifest. Reports go to stdout; `--json` selects JSON.
Use Cargo's release profile for full-corpus processing and ingest performance
measurements, including builds that reuse every embedding. Source preparation,
tokenization and publication validation remain substantial CPU work without
inference; ordinary debug builds are intended for development checks.

```bash
cargo run --release -p atlas-dev -- source analyze --source vendor/pf2e --json
```

Setup and index build/check/validate remain operational product commands:

```bash
cargo run --release -p atlas-cli -- index build --no-embeddings --json
```

For timings, build the executable first and measure its invocation separately
from compilation. Keep fresh builds, unchanged updates and content-policy upgrades
as separate cases; capture `--progress always` alongside the JSON report. Use an
isolated workspace copy for experiments, retain verified embedding checkpoints,
and avoid concurrent builds or benchmarks that would distort the comparison.

Foundry compiler research runs through the separate private
[TypeScript package](./dev-tools/source-contracts/README.md), with npm entry points.
Rust developer commands do not launch TypeScript tools. Neither developer
surface is included in published product bundles; atlas-dev is built locally.

## Release Process

User-facing binaries are published from GitHub Releases. Maintainers need the GitHub CLI and cargo-dist:

```bash
gh auth login
gh auth status
cargo install cargo-dist --locked
```

Release tooling also requires Node 22 or later and an XZ-capable `tar`. Install the
private maintainer package with `npm --prefix dev-tools/release ci --ignore-scripts`
and validate it with `npm --prefix dev-tools/release run verify`. See
[release tooling](./dev-tools/release/README.md) for notices, manifest, checksum and
platform smoke-test commands. These dependencies stay out of product bundles.

### Nightly publication

`.github/workflows/nightly.yml` runs daily at 09:23 UTC and can be dispatched
manually from `main`. Scheduled runs skip the commit already served by the nightly
channel; manual runs rebuild it. A single source SHA is used for CI, packaging and
publication. Runs from other branches do not publish.

The workflow calls the existing CI and release-build-check workflows. Nightly
packaging edits only the CI checkout's CLI manifest and lockfile, using a version
such as `0.1.0-nightly.20261018092300.gabcdef123456`. No version-bump commits or
handwritten release notes are needed. The four-platform packaging checks run the
actual binaries before upload. The publication job validates all assets, publishes
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
core.editor` after creating the file. Edit the release notes, validate, commit,
and open the PR to `main`.

2. After editing the release notes, have the helper validate, commit, push, and
   open the release-preparation PR:

```bash
scripts/prepare-release.sh --open-pr
# or: just release-open-pr
```

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

## Validation At Commits And Review

Before each logical commit, inspect the scoped diff and run checks appropriate to the change. Before marking a PR ready for review, run the full applicable gates. A draft PR may be opened earlier with outstanding validation clearly identified. Respect the tracked hooks; do not bypass them. Reuse successful validation for an unchanged candidate rather than manually repeating expensive checks without a reason.

For Rust changes, run the full gate with the compiler pinned in
`rust-toolchain.toml`. Rustup-managed Cargo
selects that toolchain automatically; standalone installations such as Homebrew
use their own compiler. Check `rustc --version` and put the pinned toolchain on
`PATH` when necessary. Validation with a different compiler is separate evidence.

```bash
just verify
# or: scripts/verify.sh
```

Use `just verify --verbose` or `scripts/verify.sh --verbose` to stream detailed
Cargo output for every successful gate.

During frontend iteration and before pushing, run the fast checks and focused
behavioral tests. These catch invalid test APIs and type errors without waiting
for the complete suite:

```bash
just web-ui-check
npm --prefix web/atlas-ui run test -- src/features/encounters/encounterViews.test.tsx
```

Run focused tests for the actual changed owner; the encounter example is not a
universal smoke test. At a coherent final candidate, run the full frontend gate:

```bash
just web-ui-verify
# or: npm --prefix web/atlas-ui run verify
```

Shared browser layout, navigation, overlays and forms also use the maintained
device acceptance suite:

```bash
npm --prefix web/atlas-ui exec -- playwright install chromium webkit
npm --prefix web/atlas-ui run test:devices
```

Concurrent worktrees must use separate device fixture ports, for example
`ATLAS_DEVICE_PORT=18867 npm --prefix web/atlas-ui run test:devices`. The suite
checks the served frontend against that checkout's build before every test.


The command builds the frontend and launches `atlas-web`'s developer-only
`browser_fixture` example on 127.0.0.1:8796. The first run compiles Rust using the
normal checkout-local target; later runs reuse it. Port 8796 must be free: the
suite never adopts an unrelated running server. It creates a small checked-source
persistent SQLite workspace under `scratch/responsive-validation/runs`.
Graceful SIGINT removes the default temporary run directory. A forced kill may
leave a run directory for inspection; remove only that inactive run after retaining
needed evidence. No Foundry clone or embedding models are downloaded.

The fixture uses real index, retrieval, app-service and web owners. Structural
stored vectors enable browsing/Similar without claiming inference or ranking
quality; query-asset unavailability remains truthful. The bounded Chromium/WebKit
phone/tablet/desktop matrix exercises real touch and keyboard flows. It is separate
from `npm --prefix web/atlas-ui run test:e2e`, whose broader scenarios require a
caller-supplied real corpus server.

CI builds the frontend once after fast checks. Full Rust verification and a
browser-fixture build then run independently. Two browser shards reuse that
compiled fixture; each runner starts its own disposable SQLite workspace and one
Playwright worker. No mutable database or live Cargo target is shared between
runners. The
`validation complete` check requires both shards, the fixture build, the frontend
checks/build, unit tests and every other selected gate. Main/nightly retain all
surfaces. Frontend `verify` alone does not execute browsers. Browser dependencies
use the Ubuntu HTTPS archive, with a three-minute installation limit; an installer/network failure can rerun just its browser
job with `gh run rerun RUN_ID --job JOB_ID`, reusing the retained compiled fixture.
Use the `databaseId` from `gh run view RUN_ID --json jobs` as `JOB_ID`.

The fixture artifact contains the executable, its expected frontend index and a
manifest with source commit, platform, architecture and SHA-256 checksums. Set
`ATLAS_BROWSER_ARTIFACT` to that directory to run it without Cargo. The runner
rejects another commit/host or corrupted files; every test also checks served
HTML against that artifact's index. CI artifacts expire after three days. Rebuilding the same candidate replaces its
intermediate bundle/fixture; report artifacts include the run attempt so failure
evidence remains available. Local
runs default to the checkout-local Cargo build. Never point a candidate test at a
frozen preview binary just because it starts successfully.

Rust verification writes a dependency-only cache shared with the fixture build;
the fixture job only restores it. The action keys compiler identity, lockfiles,
Cargo configuration and relevant environment. Workspace crates remain rebuilt,
CI incremental compilation stays disabled, and no Cargo target is transferred as
a test artifact. GitHub's branch scoping and eviction bound retained caches.
Compare job/step timings on cold and warm runs before claiming a speedup.

### Focused Linux browser diagnosis

After a failed device run, repeat the failing spec/project against that run's
compiled candidate instead of rerunning all gates:

```bash
gh workflow run browser-debug.yml --ref feat/my-branch \
  -f browser-run=RUN_ID -f file=turn-reminders.spec.ts \
  -f project=phone-webkit -f grep='recovery' -f repeat=5
```

Use a run from this repository with an unexpired `browser-fixture` artifact. The
manual run checks out the artifact's exact source commit and uses the same Ubuntu
24.04, Node 22 and lockfile-pinned Playwright installation as ordinary CI. It runs
only the selected case, with traces/screenshots/report retained even on failure;
it does not replace the full PR check or emit its `validation complete` result.
The `browser-debug.yml` workflow must be registered on the default branch before
GitHub permits direct dispatch. It owns read-only Actions access for replay;
main/nightly verification keeps its own permissions and checks. For local diagnosis, pass the
same spec, `--project`, `--grep` and `--repeat-each` arguments to Playwright directly.

### Visual references

Device runs always compare the condition refinement checkpoints with tracked
references under `web/atlas-ui/e2e/devices/visual/<platform>/<project>/`. Exact
pixels, viewport, theme and fixture state are preserved; interaction tests also
check focus, geometry, retained anchors and delayed transitions. References are
platform-specific. Missing or changed references fail ordinary runs, and CI
uploads actual/expected/diff images and traces for review. Local evidence lives
under `scratch/responsive-validation/browser`.

To propose Linux references deliberately, dispatch the focused runner with
`-f file=condition-refinement.spec.ts -f project=PROJECT -f repeat=1 -f capture=true`.
Use `-f project=all` to capture every project. This captures candidate files in the evidence
artifact; it never commits them or changes the ordinary gate. Inspect the rendered
states and differences against the accepted reference before copying reviewed
PNGs into Git. For macOS use the same focused command locally with
`--update-snapshots=all`. Do not refresh references to make a refactor pass.
Record source/browser/OS provenance in the visual directory README. A deliberate
CSS perturbation should fail the comparison when verifying this guard.

Follow the frontend guidelines for visual review and actual-device checks;
passing emulation does not certify a physical phone's software keyboard, browser
chrome or text-enlargement behavior.

For path-sensitive formatting or explicit full validation (the range below assumes a PR against `main`; substitute the actual PR base for integration or stacked work):

```bash
just verify-changed --staged
just verify-changed --range origin/main...HEAD --list
just verify-changed --range origin/main...HEAD --full
```

The selector prints its decisions with `--list` without running checks. Local
`--full` dispatches to the owning validation surface; install that surface's npm
dependencies explicitly first. Rust workspace builds also require the frontend
bundle: the selector prepares it before Rust checks, reusing the frontend gate’s
build when both surfaces are selected. It never installs dependencies as part of
a gate.

| Changed surface | Local full validation / PR CI coverage |
| --- | --- |
| Rust | Workspace gate; CI also checks Windows storage and standalone atlas-dev |
| Frontend | Frontend gate; CI also runs Rust verification and real-app browsers |
| App-model contracts | Both Rust and frontend |
| Foundry model | Rust and source-contract generation |
| Source-contract tooling | Fixture tests and pinned generation |
| Release tooling | Tool tests, notices and installer checks; CI includes three OSes |
| Research datasets, including hash-bound Markdown | Each dataset's offline `verify.ts`, with no inference |
| Prose documentation | Diff/links/examples review; no application builds |
| Cargo manifests/lockfile | Rust, frontend, source contracts and release/notices |
| Shipped agent skill | Metadata and bundled-file coverage checks (`node scripts/validation/skill.ts`); no application build |
| Unknown paths, workflow or validation infrastructure | All surfaces |

PR CI uses the same path classifier. Renames include the old and new paths;
executable/symlink mode changes select all. Main pushes and nightly verification
run every surface. The `validation complete` job rejects failed or unexpectedly
skipped selected jobs; use it as the aggregate required check. CI selection and
local script stages report decisions/timings in the job summary. Two Clippy passes
retain distinct lint policies; tests and normal builds remain separate until
measurement establishes equivalent coverage at lower cost.

`test:devices` builds the frontend for standalone local use. CI invokes Playwright
directly against the authenticated fixture artifact, without rebuilding Rust or
the frontend in each shard.
`just verify-all` validates installed Rust/frontend dependencies without rerunning
`npm ci`; use `just web-ui-install` when the lockfile or installation changes.

Tracked git hooks live in `.githooks/`:

- `pre-commit` and `pre-merge-commit`: check staged whitespace and run path-sensitive formatting checks (Rust and/or frontend). Docs-only changes need only the whitespace check.
- `commit-msg`: require a Conventional Commit subject; an optional body must be blank-line-separated.
- `pre-push`: check third-party notices when outgoing commits change Cargo manifests/lockfiles, notices, Cargo configuration, or release tooling. New refs are checked conservatively. This rebuilds the small TypeScript release tool and runs Cargo metadata offline; it does not compile Rust, install packages, or rewrite notices. Install dependencies explicitly with `npm --prefix dev-tools/release ci --ignore-scripts`. For stale notices, run `npm --prefix dev-tools/release run notices` and commit the result. Push relevant refs from a checkout at their tip with committed dependency/notices inputs so the check validates the published candidate.
- Run the applicable local gates explicitly before marking a PR ready, reuse passing evidence for unchanged code, and require green CI on the published head before merging. CI retains Clippy, typechecking, tests, builds and platform/browser checks.

Formatting checks read files in the working tree; whitespace checks read the staged diff. Review partial staging carefully so local evidence corresponds to the committed candidate. Hooks do not replace code review or CI.

Domain-owned SQL is the physical schema source of truth: reference SQL lives in `crates/atlas-index/migrations/`, saved-domain SQL in `crates/atlas-local-state/src/schema.sql`. Runtime composes the ordered workspace migration ledger; storage executes it with checksums, backup and atomic validation. Once a migration is released, append an upgrade rather than editing its applied SQL. Keep tests for failed-upgrade rollback and saved-state preservation. Query-only changes can use the supported DTO-based projection backfill; do not force source admission or inference for schema changes alone.

Docs-only or instruction-only changes do not require the Rust or frontend gates unless they affect executable examples, generated output expectations, or another validated artifact. Inspect the diff and check whitespace and any changed links or examples.

The normal landing path is a reviewed GitHub PR, with applicable local gates, CI, and human acceptance reported separately. `scripts/land-worktree.sh` and `just land-worktree` directly fast-forward local `main`; they are not the normal PR workflow and require explicit authorization to use. Do not interpret a passing gate as permission to merge.

## Pinned PF2E Data

`atlas setup --path-mode repo` acquires the model-aligned revision into
`.cache/pf2e/<commit>`. The canonical pin is
`dev-tools/source-contracts/source-pin.json`; runtime identity is generated from it.
Global mode uses `pf2e-atlas/pf2e/<commit>` under the platform cache directory.
The cache is detached and immutable: local edits or a different revision are
reported, never overwritten. Cold acquisition requires Git and network access;
warm check/offline use does not fetch.

A separate `vendor/pf2e` checkout may be used for developer diagnostics or passed
explicitly with `--source`. Atlas never pulls or checks out supplied directories.
Existing workspaces retain their saved source selection. To move one to the
managed repo-mode source without touching the old checkout or saved state:

```bash
atlas setup --path-mode repo --source .cache/pf2e/4cbdaa37d6c33e9519561bae2c59a23e0288cbce
```

PF2e system sources must declare the supported version for product builds.
Custom module versions are not interpreted as PF2e versions. Developer
`atlas-dev source load` remains available for diagnosing unsupported inputs.
Changing the pin requires regeneration and corpus/consumer validation; it is not
an ordinary data refresh. See ADR0052.

## Source declaration research

The shared HTML interpretation API is in `atlas-record::source_content`. Its
tests include typed generated-source traversal with an owned Item and invalid
neighboring field. The developer example can prepare an explicitly inventoried
HTML-field JSONL sample with supplied localization and a simple root-record
lookup index:

```bash
cargo run -p atlas-record --example source_content_probe -- HTML-fields.jsonl locale.json records.jsonl output.jsonl
```

Input field packets contain `owner` (`pack.id`, optionally with an owned suffix),
`field` (dotted source field) and `markup`; record-index packets contain `id`
(`pack.id`) and `name`. These are probe inputs, not product CLI or artifact
contracts. The probe explicitly uses source-inspection audience settings and
snapshot-local child locators. The output is not a browser/Foundry acceptance
result or an audience-safe application response. See
[ADR 0044](./docs/architecture/decisions/0044-shared-source-content-interpretation.md).

Offline declaration and trait metadata tooling lives in [dev-tools/source-contracts](./dev-tools/source-contracts/README.md). Its pinned Node dependencies, strict TypeScript build and fixture tests are separate from the Rust build and published product. Run `npm --prefix dev-tools/source-contracts ci --ignore-scripts`, then `npm --prefix dev-tools/source-contracts run verify`. Run extraction with `npm --prefix dev-tools/source-contracts run extract -- --source PATH --out PATH [--strict]`. Rust developer commands use Rust libraries; compiler research runs through this npm package.

Source-model generation also requires rustfmt. Package verification
uses small offline fixtures. `verify-generated` extracts the pinned upstream
declarations and checks committed Rust, fetching into the ignored cache when needed.
`generate` emits/checks the complete authored portfolio, `sample-equipment`
extracts raw equipment packets and `compare-equipment` compares Rust probe results.
`sample-items` recursively samples Item sources, including embedded/subitems;
the Rust `item_generation_probe` example compares typed slices against raw source
projections. These private experiment commands are documented in the package README and
[shared Item report](./docs/research/shared-item-source-generation.md). Rust tests
live under atlas-foundry-model; its source parsers need no Node runtime.
`sample-predicates` extracts selected predicate-bearing field contexts, including
ChoiceSet constructor inputs. The Rust `predicate_generation_probe` compares
typed values and reports source conflicts with a nonzero exit. See the
[recursive generation report](./docs/research/recursive-source-generation.md).
`sample-rules` samples root/embedded Item rules; `compare-rules` compiles scratch
schema/authored portfolios and checks raw-value fidelity in Rust. Rejections and
unmodeled keys remain counted and exit nonzero. Foundry runtime admission is a
separate evidence state; see the [authored-rule report](./docs/research/authored-rule-source.md).

The small `dev-tools/source-contracts/source-pin.json` records upstream identity.
Private `npm --prefix dev-tools/source-contracts run generate -- --check` fetches
the pin when needed, extracts afresh into ignored `.cache/source-contracts`,
and checks committed Rust under `crates/atlas-foundry-model/src/source_model/generated`.
Omit `--check` to regenerate. `--source PATH` seeds the cache from matching local
source without modifying that checkout. CI runs `verify-generated` separately
from offline fixture tests. Ordinary Rust builds require neither Node nor Foundry.

`atlas-foundry-model` owns pure strict/admission APIs and typed snapshots. Ingest
owns files/provenance and calls shared admission once. Model serialization
includes explicit union tags. `parse_source_value` reads authored JSON, while
Serde Deserialize reads typed source values; never use it to admit source files.

Run the standalone corpus proof against the authenticated source pin:

```sh
cargo run -p atlas-foundry-model --example snapshot_corpus -- /path/to/pinned/pf2e
```

It asserts exact Rust equality after encoding/decoding every root and checks
pinned loading outcomes. It is contributor validation, not runtime loading or
an artifact migration. Codec/source policy changes follow ADR 0043.

The database-independent record enrichment handoff is
`atlas-ingest::enrich_loaded_source`. It consumes admitted models, retains source
bytes/provenance and diagnostics, and calls `atlas-record::source_record` for
owned identity, selected content, references and typed query/text views. Audience
and implicit check-DC policy are required inputs; no product default is implied.

Run its bounded corpus evidence example with the authenticated source pin and
that pin's locale:

```sh
cargo run -p atlas-ingest --example source_enrichment_probe -- /path/to/pinned/pf2e /path/to/pinned/pf2e/static/lang/en.json
```

The JSON report checks all retained typed snapshots before/after enrichment,
compares selected query values independently to authored JSON, checks visible
reference/interaction markers, and measures sizes and stage/warm timings. Raw
JSON comparison exists only in this developer proof. The probe explicitly uses
a public audience with GM-only implicit DCs; it does not publish CLI commands,
persist corpus receipts or validate browser behavior, search relevance or SQLite
hydration. See [ADR 0045](./docs/architecture/decisions/0045-source-backed-record-enrichment.md)
and the [projection inventory](./docs/research/source-record-query-projections.md).


`compare-portfolio` checks these maintained models against all five document
kinds and specific rules through the Rust `source_portfolio_probe` example.
It reports every rejection and value difference, and compares the pinned corpus
baseline. Known failures still return nonzero, even when the baseline matches;
this tooling does not decide production ingest rejection policy. See the package
README and [maintained portfolio report](./docs/research/maintained-source-portfolio.md).

Search-profile operations persist requested configuration independently of the
active reference manifest: fresh `atlas setup` prepares BGE Base; omitted flags
preserve requested profiles, and explicit `--search-profile` flags add to them. Search/web omission selects a sole stored
profile, or Base when both are built. Pinned model files live under
`hf-models/<model-id>/<revision>/`; setup repairs selected invalid files through
verified temporary download and rename. Do not relocate this path policy into
vector identity or add a legacy-layout reader.

Use `--progress always --json >report.json 2>progress.log` when reviewing normal
build progress. Stages emit concise operation labels and structured counters/units;
the CLI renderer owns counts and elapsed/estimated time. Keep paths, revisions,
reuse statistics and internal terminology out of normal progress text; use
`ATLAS_LOG=info` for diagnostic fields. Check selected model names, passage versus
unique-passage counts and bounded stage updates. ETA is approximate, omitted until
there is a useful sample, and rounded to minutes for long-running stages. Include an unchanged
reuse-heavy tiny build and `--progress never`; JSON stdout must stay parseable.
Rendering/control-plane corrections do not justify a full-corpus reembedding.

### Embedding storage regression probe

Normal setup/build reclaims substantial free SQLite pages after publication. An
ordinary startup may migrate the schema but does not vacuum; run `atlas setup`
explicitly to reclaim space after a storage upgrade. A failed reclamation retains
complete reference data and can be retried without repeating inference.

For the schema-9 to schema-10 storage change, an optional corpus probe captures
one consistent disposable snapshot, verifies every vector and source attribution,
compares saved-state hashes and filtered distances, checks exact-input reuse and
runs deep validation. It never migrates its source. Use a repository development
corpus, with enough temporary space for the snapshot, normal migration backup and
SQLite staging; all copies are removed on completion:

```bash
ATLAS_STORAGE_PROBE_SOURCE=/absolute/path/to/development/workspace.sqlite \
  cargo test -p atlas-dev --test embedding_storage -- --ignored --nocapture
```

These local debug timings are diagnostic, not release performance guarantees.
Exact-distance KNN ties may return different witnesses after packing; comparison
checks the distance sequence rather than arbitrary tied IDs.
