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
- `crates/atlas-cli`: product command parsing, presentation, exit codes, and agent skill installation
- `crates/atlas-dev`: private Rust developer source and artifact diagnostics
- `crates/atlas-cli-support`: shared argument vocabulary, JSON envelopes, and progress rendering
- `crates/atlas-runtime`: path resolution, setup readiness, source-fetch policy, and runtime handle construction
- `crates/atlas-search`: product-facing retrieval orchestration
- `crates/atlas-index`: SQLite artifact schema/migrations, validation, row readers, artifact writing, filter discovery, filter compilation, and vector SQL
- `crates/atlas-foundry-model`: generated authored DTOs, field-retaining admission and checked snapshot codecs
- `crates/atlas-ingest`: typed Foundry loading, preparation, attributed indexing inputs, embedding execution and artifact-build composition
- `crates/atlas-embedding`: the pinned BGE model contract, tokenizer budgets, library inference and attributed passage input preparation
- `crates/atlas-record`: source-backed records, borrowed authored views, selected content preparation, query facts and reference policy
- `crates/atlas-domain`: shared typed query, source address, lightweight summary and canonical record-key vocabulary
- `crates/atlas-app-service`: saved-list and encounter workflows, local overlays and consumer presentation projections
- `crates/atlas-app-model`: generated frontend contracts for product workflows and presentation
- `crates/atlas-local-state`: saved lists and encounters in a separate writable database
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
`git push` directly; its validation hook needs no Cargo environment override.

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
cargo run -p atlas-dev -- index inspect --index .cache/pf2e-index.sqlite --json
cargo run -p atlas-dev -- index record actionspf2e:1kGNdIIhuglAjIp9 --index .cache/pf2e-index.sqlite --json
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
Use Cargo's release profile for ingest performance measurements:

```bash
cargo run --release -p atlas-dev -- source analyze --source vendor/pf2e --json
```

Setup and index build/check/validate remain operational product commands:

```bash
cargo run -p atlas-cli -- index build --no-embeddings --json
```

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
- `pre-push`: run path-sensitive full checks for pushed Rust and web UI changes

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

## Source Modeling Integration

Source-modeling work lands through `integration/source-modeling`. The bottom
feature PR targets that branch; dependent PRs target the preceding feature branch.
The draft integration-to-`main` PR shows work as reviewed feature PRs land and
remains a separate merge decision. `integration/record-refactor` is retained as
the earlier research and product-preview reference.

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
