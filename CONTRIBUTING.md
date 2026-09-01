# Contributing

## Branching

This repo uses a simple trunk-based workflow:

- `main` is the primary branch
- create short-lived feature branches from `main`
- merge back to `main` after validation

Recommended branch names:

- `feat/<topic>`
- `fix/<topic>`
- `docs/<topic>`
- `chore/<topic>`

## Commit Messages

Use Conventional Commits for all changes. Every commit message must include a Conventional Commit summary line. A short description body is optional, but when present it must appear after a blank line.

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
- `crates/atlas-cli`: command parsing, JSON/text output, progress output, exit codes, and agent skill installation
- `crates/atlas-runtime`: path resolution, setup readiness, source-fetch policy, and runtime handle construction
- `crates/atlas-search`: product-facing retrieval orchestration
- `crates/atlas-index`: SQLite artifact schema/migrations, validation, row readers, artifact writing, filter discovery, filter compilation, and vector SQL
- `crates/atlas-ingest`: Foundry source loading, normalization, enrichment, generated records, embeddings during builds, and SQLite artifact writing
- `crates/atlas-embedding`: model catalog, query/document embedding generation, token budgeting, and semantic input rendering
- `crates/atlas-record`: normalized records, content documents, presentation, FTS projection, and graph/reference policy
- `crates/atlas-domain`: shared request, filter, record-key, detail-level, and metadata vocabulary
- `crates/atlas-sqlite-vec`: sqlite-vec registration and capability probing
- `skills/pf2e-atlas-cli`: first-party local-agent skill installed by `atlas agent skills`

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

Build and test from the repository root:

```bash
just verify
# or: scripts/verify.sh
```

Validation is quiet by default: successful gates print summary lines, and detailed
Cargo output is replayed only when a gate fails. Use `just verify --verbose` or
`scripts/verify.sh --verbose` when you want the full command stream.

Validation has four explicit tiers:

- `just validate-fast` is the focused formatting plus complementary Clippy
  tier: strict panic-oriented denies apply to runtime libraries/binaries, while
  tests, benches, and examples retain warnings-as-errors and the `dbg!` ban.
  The target sets do not overlap and no all-targets pass repeats runtime linting.
  It does not run workspace tests/build or corpus/deep validation.
  `scripts/validation/fast.sh --base <ref>` makes it path-sensitive and also
  runs the merge-base artifact-version policy guard.
- `just validate-focused` runs the ingest/index source-contract, mutation,
  corruption, publication, generation-binding, and validation-snapshot tests.
  It also removes corpus identity variables and never scans the full source.
- `just validate-exhaustive --source <path> --candidate-head <sha>
  --snapshot-root <new-path> --report <new-path>` is the author acceptance gate.
  Set `PF2E_EMBEDDING_CACHE_ROOT` or pass `--embedding-cache-path`. The command
  requires clean source and candidate checkouts, performs exactly one source
  traversal, and feeds analysis, strict audit, canonical closure, and both
  artifact modes from the captured in-memory state.
- `just validate-exhaustive-review` is reserved for an independent reviewer. It
  forces a new snapshot root and refuses reuse of author evidence.

The exhaustive snapshot is private, disposable validation evidence. Its identity
binds source/candidate commits and trees, source signature and pack manifest,
policy/contracts/schema/migrations/inventory, target/features/toolchain, and
embedding identity. Missing, dirty, partial, corrupt, ambiguous, mismatched, or
concurrently published state fails closed. The snapshot is never a runtime input,
product artifact, fallback, public serialization contract, or canonical model.
The strict audit replays its accepted pre-localization observation transform from
raw records already captured by the single source traversal; product normalization
continues to use localization. Before either success or failure is returned, the
full strict report is atomically persisted and checksum-bound with path, record,
member, destination, state/type/value, multiplicity/order, enforcement, and
closure detail. A failed exhaustive run atomically preserves that report in its
failed snapshot rather than reducing it to aggregate counts.

Within each exhaustive artifact mode, validation opens one generation-bound
`SqliteIndexReader` and retains its existing Diesel hydration and rusqlite
validation connections. One explicit deep-validation result feeds inspection
and evidence projection; the same reader performs captured-record round-trip
hydration. Existing locally published generations are not rehashed on ordinary
open; transfer, copy, and recovery boundaries retain exactly-once checksums.
Per-operation timing, byte, reader/connection, hash-category, validation, and
copy counters report actual operations.

`artifact_contract_version`, `schema_version`, and `manifest_version` respectively
cover incompatible canonical/artifact semantics, physical DDL, and envelope
shape. Pull-request CI runs the deterministic merge-base guard in
`scripts/validation/check-artifact-version-bump.sh`; it inventories the actual
normalization, writer, metadata, DDL, and envelope owners and requires the exact
next version for every affected class. Its small fixture suite covers missed
owners, renames/deletes, downgrades, overlapping owners, and unrelated paths
without building an artifact. Pre-push runs the same range guard as non-blocking,
bypassable feedback; required CI remains the pre-merge authority.

Requested embedding selectors are resolved through the existing embedding-model
catalog before source traversal. Validation binds the typed model and canonical
provider ID, so accepted aliases do not create different receipt identities;
artifact metadata is still checked after publication. Any later validation
failure atomically preserves a typed `failure.json`, partial `timing.json`, and
checksum closure. When a live generation already supplies trusted visible and
generation digests, failure preservation reuses them and does not rehash the
artifact pair.

Legacy/new matrix reproduction is a one-time trust-establishment review for this
migration only. Once an independent reviewer proves equivalence, permanent
candidate acceptance and CI use only the consolidated fast, focused, exhaustive,
and reviewer tiers with change-sensitive exhaustive triggering; the migration
matrix is not a permanent recipe, normal CI gate, or Checkpoint C rerun.

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

## Release Process

User-facing binaries are published from GitHub Releases. Maintainers need the GitHub CLI and cargo-dist:

```bash
gh auth login
gh auth status
cargo install cargo-dist --locked
```

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

## Validation Before Commit

Run the full Rust gate before opening a branch for review, merging back to `main`, or preparing a Rust-heavy commit manually:

```bash
just verify
# or: scripts/verify.sh
```

Use `just verify --verbose` or `scripts/verify.sh --verbose` to stream detailed
Cargo output for every successful gate.

For web UI changes, run the frontend gate:

```bash
just web-ui-verify
# or: npm --prefix web/atlas-ui run verify
```

For the same path-sensitive validation used by hooks:

```bash
just verify-changed --staged
just verify-changed --range origin/main...HEAD --full
```

Tracked git hooks live in `.githooks/` and enforce:

- `pre-commit`: run path-sensitive fast checks for staged Rust and web UI changes; docs-only commits are allowed without code validation
- `commit-msg`: require a Conventional Commit subject line; bodies are optional but must be blank-line-separated when present
- `pre-merge-commit`: run the same path-sensitive fast checks for non-docs merge commits
- `pre-push`: report advisory merge-base artifact-version feedback, then run
  path-sensitive full checks for pushed Rust and web UI changes; Git's
  `--no-verify` bypass remains available and required CI is authoritative

When changing the SQLite artifact schema, update the Diesel migration under `crates/atlas-index/migrations/`, regenerate or edit the checked-in `crates/atlas-index/src/schema.rs` to match, and run `cargo test -p atlas-index schema_freshness`. The migration is the physical schema source of truth; the freshness test prevents `schema.rs` from becoming a second drifting table descriptor.

Fast-forward merges do not run Git's `pre-merge-commit` hook. Land linked worktrees with:

```bash
scripts/land-worktree.sh
# or: just land
```

Run that command from the linked task worktree. It rebases onto `main`, runs the Rust verification gate, fast-forwards `main`, and reruns the same gate on `main`.

## Vendored PF2E Data

This repo expects a separate PF2E checkout under `vendor/pf2e`.

Initial clone:

```bash
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
```

The normal setup path can fetch or update source data automatically:

```bash
atlas setup
```

### Source-Faithful Record Workflow

Checkpoint B approved the source-faithful contract in ADRs 0033-0036. Implementation remains dependency-ordered and creature-first; approval did not authorize later family implementation, deployment, or skipping the named task owners. Contributors changing source interpretation or canonical records must keep these steps in the same bounded task ownership:

1. Refresh the pinned PF2e source identity and regenerate/reconcile the union-derived type registry. Preserve registration-only zero-count entries and exact parent contexts.
2. Update real-owner source coverage declarations and field-level fixtures. Preserve `Missing | Null | Value` where the pinned contract permits it; zero and false are meaningful values. Empty strings, nulls, and empty collections are scaffolding for path-warning purposes, but the B1 typed boundary still validates their declared presence and shape.
3. Run focused ingest/record tests and the strict coverage gate. New meaningful unknowns, type drift, parent-context drift, lost assignment, consumed-path regression, and fixture drift must fail rather than fall through raw JSON pointers. Implemented NPC declarations are leaf-exact; add a mutation test whenever a formerly covered parent could conceal a new or stale child. The full-corpus gate retains all 313 reviewed type/role/parent-context assignments; non-creature paths remain bound to their exact H1-H11 future owners and are not treated as implemented.
4. For artifact changes, land the migration/version, checked-in Diesel schema, writer, complete `atlas-index::read` hydration, validation/inspection, corruption fixtures, CLI diagnostics, and source-normalized/artifact-hydrated equality as the one serialized C1 unit.
5. Run `just verify`; run `just web-ui-verify` for frontend-affecting work. Browser automation proves semantics/accessibility/runtime behavior only; Checkpoint E remains the separate human visual gate.
6. Search for residual raw-runtime parsing, duplicate source interpretation, partial hydration, fallback adapters, and old/new presentation paths before calling a refactor complete.

Run the relaxed audit for an aggregate local review:

```bash
atlas index audit-source-paths \
  --source vendor/pf2e \
  --limit 1000000 \
  --json
```

Run strict mode in CI and before a source refresh. To review a vendored-source update, retain the previous full report and pass it as the baseline; strict mode fails on added, removed, or reclassified meaningful paths until the diff and declarations are reviewed:

```bash
atlas index audit-source-paths \
  --source vendor/pf2e \
  --limit 1000000 \
  --strict \
  --baseline previous-source-coverage.json \
  --json
```

The JSON path list, diagnostics, source diff, disposition summaries, and pinned-base retrieval-predicate inventory are deterministically sorted. `consumed`, `ignored_with_rationale`, `provenance_only`, `deferred`, and `unknown` are distinct outcomes; every row exposes its matched rule, owner family, fixture/checkpoint, and validation contract, every consumed row names its extractor, and every deferred row names an exact future owner and plan. Strict summaries also expose generic deferred matches, unowned recursive matches, and consumed regressions; all three must be zero for a passing corpus audit.

Atlas currently has no authentication or viewer authorization boundary and is primarily a GM tool, but the pinned base still contains default-visible/public-only routing and is not GM-complete. The approved target preserves typed visibility, role, source kind, and provenance while removing classification-only suppression of useful authored information across ingest, artifact, search, graph, discovery, metrics, CLI, app, and UI. Any retained exclusion needs a documented non-auth product rationale, fixtures, validation, and audit checkpoint. Do not describe either the base predicates or target metadata as a privacy/security boundary; future authenticated filtering requires a separate approved feature.
