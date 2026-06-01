# Repository Guidelines

## Project Structure & Module Organization

PF2e Atlas is a Rust workspace. Core application code lives under `crates/`.

- `crates/atlas-cli`: command parsing, output, progress, exit codes, and agent skill installation.
- `crates/atlas-runtime`: path resolution, setup readiness, source-fetch policy, and runtime handle construction.
- `crates/atlas-search`: product-facing retrieval orchestration.
- `crates/atlas-index`: SQLite artifact schema/migrations, validation, row readers, artifact writing, filter compilation, and vector SQL.
- `crates/atlas-ingest`: Foundry source loading, normalization, enrichment, generated records, embedding execution during builds, and artifact writing.
- `crates/atlas-embedding`: model catalog, query/document embedding generation, token budgeting, and semantic text rendering.
- `crates/atlas-record`: normalized records, content documents, presentation contracts, FTS projection, and reference graph policy.
- `crates/atlas-discovery`: filter discovery field and value policy.
- `crates/atlas-domain`: shared request, filter, record-key, detail-level, and metadata vocabulary.
- `crates/atlas-sqlite-vec`: sqlite-vec registration and capability probing.

The first-party local-agent skill lives in `skills/pf2e-atlas-cli`. The vendored PF2E checkout is expected under `vendor/pf2e` but is not tracked in this repo.

## Architecture Docs

The architecture documents under `docs/architecture/` are part of the working source of truth for this repository, not optional reference material.

- Before making changes that affect module ownership, runtime composition, cross-crate dependencies, shared abstractions, service boundaries, search flow, artifact schema, or future TUI/tagging architecture, read the relevant architecture docs first.
- Start with `docs/architecture/overview.md`. For runtime, ingest, search, or artifact work, also read `docs/architecture/runtime.md` and `docs/architecture/artifact-contract.md`. Read the ADR index in `docs/architecture/decisions/README.md` when the change depends on durable architecture decisions.
- When a change materially alters the architecture or the intended editing guidance, update the relevant docs in the same task so they stay in sync with the code.
- Architecture-impacting changes include new or replaced facades, moved ownership between crates, new lint-enforced boundaries, major search-pipeline changes, storage-boundary changes, and future TUI or editorial workflow restructuring.
- During architectural rework, prefer direct replacement to compatibility layers, adapters, or shims. Treat shim-based transitions as tech debt by default.
- If you add a new durable architectural rule or make a non-obvious architectural choice that future editors will need to preserve, update an existing ADR or add a new file under `docs/architecture/decisions/`.
- Do not report an architecture-impacting implementation task as complete if the code and the architecture docs disagree about the intended structure.

## Documentation Positioning

Keep each documentation surface focused on its job:

- `README.md` should describe the current product, what users can do with it, and how to set it up and run it.
- `docs/architecture/` should describe the current intended architecture and durable boundaries that the codebase is expected to follow.
- `docs/backlog/backlog.md` should list only open backlog work using the active status vocabulary in that file.
- `docs/backlog/history/` should hold durable history of completed or retired backlog work.
- `CONTRIBUTING.md` should carry contributor workflow, validation commands, repo layout, and developer-facing operational guidance.
- Historical context, design evolution, tradeoff records, and "why we changed this" material should primarily live in ADRs under `docs/architecture/decisions/`.

When writing or editing docs, prefer statements of current behavior and current ownership over repo-history framing unless the document is explicitly an ADR or backlog/history note.

## Build, Test, And Development Commands

- `scripts/install-git-hooks.sh`: configure this clone to use the tracked git hooks in `.githooks/`.
- `scripts/preflight.sh`: verify the current checkout is a linked worktree on a non-`main` branch.
- `just verify` or `scripts/verify.sh`: run the Rust fmt, clippy, test, and build gate.
- `cargo run -p atlas-cli -- --help`: run the CLI from source.
- `cargo run -p atlas-cli -- setup --check --json`: inspect setup readiness from source.
- `cargo install --path crates/atlas-cli --locked`: install the local `atlas` CLI.

## Coding Style & Naming Conventions

This codebase is Rust. Keep modules single-purpose and put helpers beside the concern that owns their policy. Avoid generic `utils.rs` or `helpers.rs` modules. Promote shared helpers only when at least two existing owners need the same stable primitive and there is no clearer domain owner.

Prefer descriptive module names and explicit ownership:

- artifact schema, migrations, validation, and storage metadata belong in `atlas-index`
- runtime path/setup policy belongs in `atlas-runtime`
- SQLite reading and validation belong in `atlas-index`
- retrieval orchestration belongs in `atlas-search`
- source loading, normalization, enrichment, and writing belong in `atlas-ingest`
- presentation-neutral record/content models belong in `atlas-record`
- CLI presentation and exit behavior belong in `atlas-cli`

Use `cargo fmt` for formatting. Keep Clippy clean under the workspace gate.

## Testing Guidelines

Rust workspace tests are the primary validation surface. Add or update tests under the owning crate for behavior changes, especially around ingest, indexing, lookup, search, filter discovery, graph context, and artifact validation.

Run the Rust verification gate before committing non-docs changes:

```bash
just verify
# or: scripts/verify.sh
```

Docs-only or instruction-only changes do not require the Rust verification gate unless the change also affects executable examples, generated output expectations, or another validated artifact.

## Commit Guidelines

Use trunk-based branches such as `feat/<topic>` or `fix/<topic>`. Commit messages must use Conventional Commits and include a summary line; a description body is optional, but when present it must be separated from the summary by a blank line.

Agents should not commit automatically just because an implementation task appears complete. Treat commits as an explicit milestone. When the user asks to prepare a commit, use the `$prepare-commit` skill, run the required validators, fix reported issues through the same validation loop, and only then create the commit.

Do not batch unrelated changes into one commit, and do not commit half-finished work just to create progress snapshots. If the worktree already contains unrelated uncommitted changes, leave them untouched and commit only the files for the completed unit of work, or explicitly tell the user why a clean commit boundary is blocked.

## Refactor Completion Policy

Refactors must land as finished end-state changes, not as transitional scaffolding.

- Default to removing the old path outright and updating all call sites in the same task.
- Do not add compatibility layers, shim modules, fallback code paths, version bridges, or adapter wrappers unless the user explicitly asks for an incremental strategy.
- Do not treat a refactor as complete if it leaves behind intermediate shims, compatibility wrappers, transitional files, partial migrations, or mixed old-and-new implementations across the codebase.
- Validation for refactor work must explicitly check for leftover uses of the old implementation, leftover shim files, and any other signs of an intermediate state.

## Optional Worktree Policy

Agents may work in the current checkout unless the user explicitly asks for a worktree or the task has a concrete need for checkout isolation. Use worktrees for parallel delegated implementation, risky long-running work, preserving a dirty main checkout, or when the user wants an isolated branch.

Git commands that mutate repository state must never be run in parallel within the same repository or worktree. Serialize `git add`, `git commit`, `git rebase`, `git merge`, `git cherry-pick`, `git worktree`, and `git stash` operations.

## Configuration & Data Notes

Default repo-local data paths are `vendor/pf2e`, `.cache/hf-models`, and `.cache/pf2e-index.sqlite` when `--path-mode repo` is selected. The default global path mode uses platform cache paths under `pf2e-atlas`. If the PF2E checkout, embedding model, or artifact schema changes, run `atlas setup` or the relevant `atlas index` command.
