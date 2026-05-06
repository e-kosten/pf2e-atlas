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

Examples:

- `feat/encounter-tools`
- `fix/search-ranking`
- `docs/codex-setup`

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

Scope is optional:

```text
type: summary

description
```

Recommended types for this repo:

- `feat` for new MCP capabilities or data features
- `fix` for bug fixes or behavior corrections
- `docs` for README or contribution guide updates
- `refactor` for internal code restructuring without behavior changes
- `test` for test-only changes
- `chore` for maintenance tasks, scripts, or repo setup

Examples:

- `feat(search): add publication title filtering`
- `fix(lookup): prefer canonical action packs over macro packs`
- `docs(readme): document vendored PF2E checkout`
- `chore(vendor): add refresh script for PF2E data`

Full commit examples:

```text
feat(search): add publication title filtering

Prefer publication title matching in structured search filters and document the new behavior.
```

```text
fix(refresh): report index progress

Show progress updates during index rebuilds so long-running refreshes are easier to monitor.
```

## Project Shape

The codebase is organized around a few stable layers:

- `src/index.ts`: MCP entrypoint. Boots the application runtime and registers tool handlers.
- `src/app/`: application composition and cross-cutting services. This is where runtime assembly, ontology orchestration, and app-level storage boundaries live.
- `src/data/`: data loading and backend access over the prepared SQLite index and normalized PF2E records.
- `src/search/`: ranked search runtime, query analysis, and ranking logic shared by backend search flows.
- `src/server/`: MCP transport-facing tool registration and response shaping.
- `src/tui/`: terminal application composition, workflows, and UI-facing service adapters.
- `src/domain/`: shared domain types, categories, metadata semantics, and other low-level contracts used across layers.
- `src/tags/`: derived-tag authoring, discovery, migration, and evaluation tooling.

Architecture notes live under [`docs/architecture`](./docs/architecture/overview.md):

- [`overview.md`](./docs/architecture/overview.md): architecture landing page with subsystem diagrams, request flow, and navigation into the rest of the docs
- [`boundaries.md`](./docs/architecture/boundaries.md): lint-enforced and design-level boundaries that future editors should preserve
- [`search.md`](./docs/architecture/search.md): ranked retrieval, filters, and search backend design
- [`tui.md`](./docs/architecture/tui.md): terminal UI composition, workflows, and service seams
- [`editorial.md`](./docs/architecture/editorial.md): derived-tag editorial and migration tooling
- [`extending.md`](./docs/architecture/extending.md): where to add new tools, services, and runtime capabilities
- [`decisions/`](./docs/architecture/decisions/README.md): architecture decision records and follow-up design notes

## Development

`README.md` is the user-facing product and setup document. Keep contributor workflow, internal command surfaces, and repo-shape guidance here instead of expanding the README with developer-oriented details.

Install dependencies and build:

```bash
npm install
cd scripts && npm run install-hooks
cd scripts && npm run preflight
npm run build
```

Run the test suite:

```bash
cd scripts && npm test
```

Run the stdio MCP server locally:

```bash
cd scripts && npm run dev
```

Run the terminal workbench locally from source:

```bash
cd scripts && npm run dev:tui
```

To show opt-in TUI performance diagnostics in the footer while troubleshooting filter explorer stalls, set:

```bash
cd scripts
PF2E_TUI_DEBUG=1 npm run dev:tui
```

Run the two top-level built app surfaces from the repo root:

```bash
npm run build
npm run tui
npm run mcp
```

Developer command surfaces:

- Root `npm run`: user-facing setup and built runtime commands
- [`scripts/package.json`](./scripts/package.json): source-runner, validation, and local developer workflow commands
- [`src/tags/cli/package.json`](./src/tags/cli/package.json): derived-tag and editorial tooling commands

Example derived-tag/editorial CLI usage:

```bash
cd src/tags/cli
npm run discover-untagged-cohorts -- --category creature --family setting
```

## Validation Before Commit

Run these before opening a branch for review or merging back to `main`:

```bash
cd scripts && npm run verify
```

Tracked git hooks live in `.githooks/` and enforce:

- `pre-commit`: fail from the primary checkout or branch `main`
- `commit-msg`: require a Conventional Commit subject line; bodies are optional but must be blank-line-separated when present
- `pre-merge-commit`: rerun lint, build, and tests for non-docs merge commits
- `pre-push`: rerun lint, build, and tests

Fast-forward merges do not run Git's `pre-merge-commit` hook. Land linked worktrees with:

```bash
scripts/land-worktree.sh
```

Run that command from the linked task worktree. It rebases onto `main`, runs lint/build/test, fast-forwards `main`, and reruns lint/build/test on `main`.

## Vendored PF2E Data

This repo expects a separate PF2E checkout under `vendor/pf2e`.

Initial clone:

```bash
git clone https://github.com/foundryvtt/pf2e.git vendor/pf2e
```

Manual refresh:

```bash
npm run refresh-data
```
