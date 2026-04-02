# Repository Guidelines

## Project Structure & Module Organization

Core application code lives in `src/`. The MCP server entrypoint is `src/index.ts`. PF2E indexing and query logic lives mainly in `src/pf2e-data.ts`, with embedding support in `src/embeddings.ts` and refresh utilities in `src/refresh-*.ts`. Tests live in `tests/` and follow the source areas they cover, for example `tests/pf2e-data.test.ts`. Helper scripts live in `scripts/`. The vendored PF2E checkout is expected under `vendor/pf2e` but is not tracked in this repo.

## Build, Test, and Development Commands

- `npm install`: install dependencies.
- `npm run build`: compile TypeScript to `dist/` with `tsc`.
- `npm test`: run the Vitest suite once.
- `npm run dev`: run the stdio MCP server from source with `tsx`.
- `npm run start`: run the built server from `dist/index.js`.
- `npm run refresh-data`: pull the vendored PF2E checkout.
- `npm run refresh-embeddings`: prepare local embedding assets.
- `npm run refresh-index`: rebuild the local SQLite index.
- `npm run refresh-external`: run all refresh steps with visible progress.

## Coding Style & Naming Conventions

This codebase is TypeScript on Node.js with ESM (`"type": "module"`). Follow existing style: 2-space indentation, semicolons, double quotes, and small focused modules. Use descriptive file names such as `search-expansion.ts` and `refresh-index.ts`. Prefer `camelCase` for functions and variables, `PascalCase` for classes and types, and `SCREAMING_SNAKE_CASE` for constants. There is no dedicated formatter or linter configured, so match the surrounding file style closely.

## Testing Guidelines

Vitest is the test runner. Add or update `*.test.ts` files under `tests/` for behavior changes, especially around indexing, lookup, and search ranking. Run `npm test` locally before committing. For code that affects build output or types, also run `npm run build`.

## Commit & Pull Request Guidelines

Use trunk-based branches such as `feat/<topic>` or `fix/<topic>`. Commit messages must use Conventional Commits and include both a summary line and a description body separated by a blank line, for example `feat(search): add publication title filtering` followed by a short paragraph describing the change. Agents should commit work whenever a reasonable checkpoint is reached: after a coherent, tested unit of work is complete and the worktree can be described cleanly in one commit. Do not batch unrelated changes into one commit, and do not commit half-finished work just to create progress snapshots. When the user immediately asks for a follow-up adjustment to work that was just committed locally, prefer folding that adjustment into the same logical commit: soft-reset or otherwise rewrite the unpublished local commit, apply the requested patch, rerun relevant verification, and recommit instead of stacking a trivial fixup commit. This is a personal-project workflow with no PR step, so once a task branch is complete and validated it should be merged back into `main`. Before considering the work done, run `npm run build` and `npm test`, describe the user-facing change, and note any required refresh steps or data/index migration implications.

## Configuration & Data Notes

Default data path is `vendor/pf2e`. Startup is offline-only: it expects a current local SQLite index and prepared embedding assets. If the PF2E checkout, embedding model, or index schema changes, rerun `npm run refresh-index` or `npm run refresh-external`.
