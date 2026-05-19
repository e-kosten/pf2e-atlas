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
- `crates/atlas-index`: read-only artifact access, validation, row readers, filter compilation, and vector SQL
- `crates/atlas-ingest`: Foundry source loading, normalization, enrichment, generated records, embeddings during builds, and SQLite artifact writing
- `crates/atlas-embedding`: model catalog, query/document embedding generation, token budgeting, and semantic input rendering
- `crates/atlas-record`: normalized records, content documents, presentation, FTS projection, and graph/reference policy
- `crates/atlas-artifact`: physical SQLite table/column descriptors, contract constants, schema SQL, and vector blob encoding
- `crates/atlas-discovery`: filter discovery field/value policy
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
```

Build and test from the repository root:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Run the CLI from source:

```bash
cargo run -p atlas-cli -- --help
cargo run -p atlas-cli -- setup --check --json
cargo run -p atlas-cli -- search "low level healing spell" --family spell --limit 5
```

Install the CLI from this clone:

```bash
cargo install --path crates/atlas-cli --locked
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

Releases use a two-part flow:

1. From a clean, current `main`, create the release-preparation branch and scaffold the version bump and release notes:

```bash
scripts/prepare-release.sh --prepare-pr
```

The helper prompts for the release version, creates `release/v<version>`, updates
`crates/atlas-cli/Cargo.toml`, refreshes `Cargo.lock`, and scaffolds
`docs/releases/v<version>.md`. Pass `--version <X.Y.Z[-rc.N]>` to skip the
prompt. Edit the release notes, validate, commit, and open the PR to `main`.

2. After the PR lands, publish the release from any clean checkout:

```bash
scripts/prepare-release.sh --publish --dry-run
scripts/prepare-release.sh --publish
```

The helper switches to `main`, fast-forwards to `origin/main`, reads the
committed crate version, validates the matching release notes, and then creates
the tag and draft release. Pass `--version <X.Y.Z[-rc.N]>` only when you want an
extra guard that the committed crate version matches that exact value.

Running `scripts/prepare-release.sh` without a mode flag opens an interactive
picker for the two workflow steps. Non-interactive shells must pass
`--prepare-pr` or `--publish`. Interactive pickers use `fzf` when it is
available, with numbered prompts as the portable fallback.

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

Repository settings checklist:

- GitHub Actions is enabled.
- Tag pushes trigger workflows.
- The release workflow can use `GITHUB_TOKEN` with `contents: write` for release asset upload.
- Only maintainers can push `v*` release tags.
- Branch protection allows release-preparation PRs to land normally.

Published release assets are treated as immutable. Fix bad published releases with a new patch release and document known issues in the affected release notes. Draft releases may be corrected before publication.

Local validation covers Rust checks, `dist plan`, release-helper dry runs, installer dry runs, release-tool smoke tests, and static script checks. GitHub-hosted CI owns platform matrix validation for macOS, Linux, Windows, and ARM targets.

## Validation Before Commit

Run these before opening a branch for review, merging back to `main`, or preparing a commit manually:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Tracked git hooks live in `.githooks/` and enforce:

- `pre-commit`: run Rust fmt, clippy, tests, and build for non-docs commits; docs-only commits are allowed without the full suite
- `commit-msg`: require a Conventional Commit subject line; bodies are optional but must be blank-line-separated when present
- `pre-merge-commit`: rerun Rust fmt, clippy, tests, and build for non-docs merge commits
- `pre-push`: rerun Rust fmt, clippy, tests, and build

Fast-forward merges do not run Git's `pre-merge-commit` hook. Land linked worktrees with:

```bash
scripts/land-worktree.sh
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
