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

Use trunk-based branches such as `feat/<topic>` or `fix/<topic>`. Commit messages must use Conventional Commits and include both a summary line and a description body separated by a blank line, for example `feat(search): add publication title filtering` followed by a short paragraph describing the change. Agents must commit any coherent, validated unit of work before reporting the task complete unless the user explicitly says not to commit. Do not present implementation work as finished while your tracked changes for that task remain uncommitted. Final completion messages for implementation work must include the commit SHA and commit message. Do not batch unrelated changes into one commit, and do not commit half-finished work just to create progress snapshots. If the worktree already contains unrelated uncommitted changes, leave them untouched and commit only the files for your completed unit of work, or explicitly tell the user why a clean commit boundary is blocked. When the user immediately asks for a follow-up adjustment to work that was just committed locally, prefer folding that adjustment into the same logical commit: soft-reset or otherwise rewrite the unpublished local commit, apply the requested patch, rerun relevant verification, and recommit instead of stacking a trivial fixup commit. This is a personal-project workflow with no PR step, so once a task branch is complete and validated it should be merged back into `main`. Before considering the work done, run `npm run build` and `npm test`, commit the validated changes, describe the user-facing change, report the commit SHA and message, and note any required refresh steps or data/index migration implications. If build or test verification fails, do not commit unless the user explicitly asks for that state to be committed.
- Before committing, validate the completed work against the discussed plan for the task. If any agreed plan item was deferred, partially implemented, or dropped, call that out explicitly and do not present the task as fully complete.

### Agent Worktree Policy

Agents must do implementation work in a dedicated git worktree, not in the shared main checkout.

- Before editing tracked files, create or switch to an isolated worktree rooted at a new branch from the current `main` HEAD.
- Create agent worktrees only under a writable sandbox root available to the current session, such as the literal path `/tmp`. Use the literal writable-root path string itself, not a symlink-resolved equivalent such as `/private/tmp`, even if both point to the same location on the host.
- Prefer a deterministic worktree path directly under that writable root, for example `/tmp/pathfinder-mcp-worktree-<task>`. Do not place agent worktrees anywhere else unless the instructions for the current environment explicitly name another writable root.
- Do not create sibling worktrees next to the main checkout unless that path is explicitly writable in the environment.
- Do not share a checkout with another running agent, and do not reuse the user's current working tree for agent edits.
- Git commands that mutate repository state must never be run in parallel within the same repository or worktree.
- Do not use parallel tool execution for `git add`, `git commit`, `git rebase`, `git merge`, `git cherry-pick`, `git worktree add`, `git worktree remove`, `git stash`, or any other git command that writes refs, the index, or worktree metadata.
- Do not use `multi_tool_use.parallel` for git commands unless every git command in that batch is strictly read-only.
- Serialize all git write operations and wait for each one to finish before starting the next.
- Treat `.git/index`, `.git/ORIG_HEAD`, `.git/refs/*`, and `.git/worktrees/*` as shared lock-producing paths.
- Never run `git add` and `git commit` in parallel.
- Never run `git status` in parallel with git write operations in the same worktree.
- Commit and validate the change inside the worktree first.
- When the worktree change is ready to land, inspect the shared `main` checkout first. If `main` has any uncommitted tracked changes, stop and tell the user instead of merging.
- Before attempting to merge back, rebase the worktree branch onto the current `main` so parallel agent commits already landed on `main` are incorporated first.
- If the rebase hits conflicts, stop there and tell the user what conflicted so they can decide the next step.
- After a clean rebase, rerun `npm run build` and `npm test` in the rebased worktree. If either fails, stop and tell the user instead of merging.
- Only after `main` is clean, the rebase is conflict-free, and the rebased worktree passes build and tests should the agent merge the branch back into `main`.
- Merging back into the shared `main` checkout can require sandbox approval because git updates shared refs such as `ORIG_HEAD`; request escalation when the environment blocks that write.
- Do not assume a fast-forward merge will be available before rebasing; parallel agent work commonly means the landing branch has diverged from `main`.
- After merging back into `main`, rerun `npm run build` and `npm test` on `main`, and only then report the task complete.
- Remove the temporary worktree after the change has been safely integrated unless the user asks to keep it.

## Configuration & Data Notes

Default data path is `vendor/pf2e`. Startup is offline-only: it expects a current local SQLite index and prepared embedding assets. If the PF2E checkout, embedding model, or index schema changes, rerun `npm run refresh-index` or `npm run refresh-external`.
