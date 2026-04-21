# Repository Guidelines

## Project Structure & Module Organization

Core application code lives in `src/`. The MCP server entrypoint is `src/index.ts`. Shared application composition lives in `src/app/`, index-backed retrieval and backend services live in `src/data/`, ranked-search mechanics live in `src/search/`, MCP tool registration lives in `src/server/`, terminal UI code lives in `src/tui/`, shared contracts live in `src/domain/`, and derived-tag/editorial tooling lives in `src/tags/`. Embedding support lives in `src/embeddings.ts`, and refresh utilities live in `src/refresh-*.ts`. Tests live in `tests/` and follow the source areas they cover. Helper scripts live in `scripts/`. The vendored PF2E checkout is expected under `vendor/pf2e` but is not tracked in this repo.

## Architecture Docs

The architecture documents under `docs/architecture/` are part of the working source of truth for this repository, not optional reference material.

- Before making changes that affect module ownership, runtime composition, cross-layer dependencies, shared abstractions, service boundaries, search flow, TUI structure, or editorial/tagging architecture, read the relevant architecture docs first.
- Start with `docs/architecture/overview.md` and `docs/architecture/boundaries.md`, then read the focused doc closest to the change, such as `search.md`, `tui.md`, `editorial.md`, `extending.md`, or the ADR index in `docs/architecture/decisions/README.md`.
- Do not treat `AGENTS.md` as a substitute for those docs. `AGENTS.md` tells you how to work in the repo; the architecture docs tell you how the system is intentionally shaped.
- When a change materially alters the architecture or the intended editing guidance, update the relevant docs in the same task so they stay in sync with the code.
- Architecture-impacting changes include new or replaced facades, moved ownership between layers, new lint-enforced boundaries, major search-pipeline changes, TUI composition changes, storage-boundary changes, and editorial workflow restructuring.
- During architectural rework, prefer direct replacement to compatibility layers, adapters, or shims. Treat shim-based transitions as tech debt by default, not as a neutral implementation choice.
- If you add a new durable architectural rule or make a non-obvious architectural choice that future editors will need to preserve, update an existing ADR or add a new file under `docs/architecture/decisions/`.
- Do not report an architecture-impacting implementation task as complete if the code and the architecture docs disagree about the intended structure.
- Large architectural work is not complete until the relevant architecture docs are updated and any required ADR additions or revisions have been made.

## Build, Test, and Development Commands

- `npm install`: install dependencies.
- `npm run install-hooks`: configure this clone to use the tracked git hooks in `.githooks/`.
- `npm run preflight`: verify the current checkout is a linked worktree on a non-`main` branch.
- `npm run build`: compile TypeScript to `dist/` with `tsc`.
- `npm test`: run the Vitest suite once.
- `npm run verify`: run the required validation pair (`build` + `test`).
- `npm run dev`: run the stdio MCP server from source with `tsx`.
- `npm run start`: run the built server from `dist/index.js`.
- `npm run refresh-data`: pull the vendored PF2E checkout.
- `npm run refresh-embeddings`: prepare local embedding assets.
- `npm run refresh-index`: rebuild the local SQLite index.
- `npm run refresh-external`: run all refresh steps with visible progress.

## Coding Style & Naming Conventions

This codebase is TypeScript on Node.js with ESM (`"type": "module"`). Follow existing style: 2-space indentation, semicolons, double quotes, and small focused modules. Use descriptive file names such as `search-expansion.ts` and `refresh-index.ts`. Prefer `camelCase` for functions and variables, `PascalCase` for classes and types, and `SCREAMING_SNAKE_CASE` for constants. ESLint and Prettier are configured in-repo; run the relevant lint/format checks instead of relying on convention alone.

When you introduce or consolidate a shared abstraction that other code should route through, add or extend lint rules to enforce that boundary once the abstraction is stable enough to be mandatory. Do not leave new shared pathways as convention-only if they are intended to replace lower-level direct usage.

## Testing Guidelines

Vitest is the test runner. Add or update `*.test.ts` files under `tests/` for behavior changes, especially around indexing, lookup, and search ranking. Run `npm test` locally before committing. For code that affects build output or types, also run `npm run build`.

- Docs-only or instruction-only changes do not require `npm test`.
- Docs-only or instruction-only changes do not require `npm run build` unless the change also affects executable examples, generated output expectations, or another validated artifact.

## Commit & Pull Request Guidelines

Use trunk-based branches such as `feat/<topic>` or `fix/<topic>`. Commit messages must use Conventional Commits and include both a summary line and a description body separated by a blank line, for example `feat(search): add publication title filtering` followed by a short paragraph describing the change. Agents must commit any coherent, validated unit of work before reporting the task complete unless the user explicitly says not to commit. Do not present implementation work as finished while your tracked changes for that task remain uncommitted. Final completion messages for implementation work must include the commit SHA and commit message. Do not batch unrelated changes into one commit, and do not commit half-finished work just to create progress snapshots. If the worktree already contains unrelated uncommitted changes, leave them untouched and commit only the files for your completed unit of work, or explicitly tell the user why a clean commit boundary is blocked. When the user immediately asks for a follow-up adjustment to work that was just committed locally, prefer folding that adjustment into the same logical commit: soft-reset or otherwise rewrite the unpublished local commit, apply the requested patch, rerun relevant verification, and recommit instead of stacking a trivial fixup commit. This is a personal-project workflow with no PR step, so once a task branch is complete and validated it should be merged back into `main`. Before considering the work done, run `npm run build` and `npm test`, commit the validated changes, describe the user-facing change, report the commit SHA and message, and note any required refresh steps or data/index migration implications. For docs-only or instruction-only changes, no build/test run is required unless the edited files themselves define a stricter expectation for that task. If build or test verification fails, do not commit unless the user explicitly asks for that state to be committed.

- Before committing, validate the completed work against the discussed plan for the task. If any agreed plan item was deferred, partially implemented, or dropped, call that out explicitly and do not present the task as fully complete.
- When work is driven by a plan file under `scratch/plans/`, validate the finished implementation against that plan file immediately before reporting success. Do not report completion if any plan item remains open, partially done, deferred, or unvalidated.
- It is acceptable to pause mid-task and check in with the user when you are blocked, need clarification, need a decision, or surface an important architectural issue. In those cases, report the current state and the blocker clearly rather than forcing the task to an artificial completion state.

### Plan Mode Policy

When operating in plan mode, the plan file is the authoritative checklist for both orchestration and completion.

- Always write plan files as new files under `scratch/plans/`. Do not overwrite or reuse an existing plan file for a new task.
- Assume plan-mode implementation is complex enough to require orchestration and sub-agent delegation by default.
- The orchestration agent should delegate both implementation and validation to sub-agents whenever practical so its own context stays focused on coordination, integration, and end-state checks.
- If implementation hits a blocker, requires user input, or exposes a consequential architectural problem, stop and check in instead of improvising around it. It is acceptable for plan-driven work to remain incomplete while waiting for that input.
- Before reporting success on work that originated from a plan file, validate the repository against that plan file and confirm that every requested item is fully implemented, fully validated, and has no remaining follow-up work hidden behind "later" steps.
- Work from a plan file is not complete until validation confirms that nothing remains to be done from the plan and that the repository is in the intended end state rather than an intermediate checkpoint.

### Refactor Completion Policy

Refactors must land as finished end-state changes, not as transitional scaffolding.

- Default to removing the old path outright and updating all call sites in the same task. Do not add compatibility layers, shim modules, fallback code paths, version bridges, or adapter wrappers unless the user explicitly asks for an incremental migration strategy.
- If you believe a compatibility layer is genuinely unavoidable, stop and explain the constraint instead of quietly introducing one. The burden is on the implementation to justify the extra layer.
- Do not treat a refactor as complete if it leaves behind intermediate shims, compatibility wrappers, transitional files, partial migrations, or mixed old-and-new implementations across the codebase.
- A refactor is only complete when the new structure has been applied everywhere it is intended to apply, the old implementation has been fully removed, and no code still depends on the replaced pathway.
- If a refactor cannot be completed cleanly without a temporary workaround or transitional compatibility layer, stop and ask for input instead of landing the intermediate state.
- Validation for refactor work must explicitly check for leftover uses of the old implementation, leftover shim files, and any other signs of an intermediate migration state. If any of those remain, the refactor is still in progress.

### Large Task Orchestration Policy

Treat a task as large when it spans multiple subsystems, requires a planned end-state refactor, introduces cross-cutting abstractions plus migrations, or otherwise cannot be completed safely as a single local edit-and-verify loop.

- For large tasks, the primary agent must act as an orchestrator first: restate the requested end state, break the work into explicit sub-tasks, identify dependencies, and define what counts as done for the overall workset.
- When sub-agents are available in the environment, large-task execution should default to delegated sub-task ownership rather than a single agent carrying the entire implementation alone. Serial delegation is acceptable when parallel work is unnecessary or unsafe, but skipping delegation for a large task requires a concrete task-specific reason.
- Each delegated or local sub-task should have a defined scope, expected artifact, and validation step so partial progress cannot be mistaken for task completion.
- Validation for delegated implementation slices should usually be delegated with the slice itself. The orchestrator should assign explicit validation ownership to sub-agents wherever practical instead of centralizing all verification locally.
- The orchestrator should keep its own context focused on coordination, integration, and end-state checks. Do not pull large validation logs or repeated test-debug loops into the orchestrator context when a sub-agent can own that verification and report back the result.
- For plan-mode work, treat the plan file as the end-state contract and require validation agents to confirm both plan completion and absence of intermediate refactor state before the orchestrator reports success.
- If a blocker or unresolved architecture question prevents a clean end-state implementation, pause the work and check in with the user instead of masking the issue with temporary code.
- Do not treat a delegated slice as done without validation evidence or an explicit explanation of what could not be validated and why. Missing validation for a slice that could reasonably have been checked means the slice is still incomplete.
- Meaningful intermediate commits inside the worktree are encouraged when they capture validated milestones, but those commits do not by themselves make the overall task complete.
- Do not describe a large task as complete, and do not merge its worktree back into `main`, until the full requested workset is implemented, integrated, and validated against the stated end-state checklist, including required architecture-doc and ADR updates for architecture-impacting work. A green intermediate slice is a milestone, not a completion signal.
- If some agreed work remains open, keep the branch in the worktree, report the remaining scope explicitly, and treat the task as still in progress even if one or more milestone commits have already been made.

### Agent Worktree Policy

Agents must do implementation work in a dedicated git worktree, not in the shared main checkout.

- Before editing tracked files, create or switch to an isolated worktree rooted at a new branch from the current `main` HEAD.
- Untracked planning artifacts under `scratch/plans/` are an exception: they may be created or edited directly in the current checkout because they are not part of git-tracked implementation work and do not benefit from a separate worktree.
- Create agent worktrees only under a writable sandbox root available to the current session, such as the literal path `/tmp`. Use the literal writable-root path string itself, not a symlink-resolved equivalent such as `/private/tmp`, even if both point to the same location on the host.
- Prefer a deterministic worktree path directly under that writable root, for example `/tmp/pathfinder-mcp-worktree-<task>`. Do not place agent worktrees anywhere else unless the instructions for the current environment explicitly name another writable root.
- Do not create sibling worktrees next to the main checkout unless that path is explicitly writable in the environment.
- Do not share a checkout with another running agent, and do not reuse the user's current working tree for agent edits.
- When a temporary worktree needs project dependencies and the dependency manifests have not changed, prefer symlinking the primary checkout's installed dependency directory into the worktree instead of reinstalling. For this repo, if `package.json` and `package-lock.json` match the primary checkout, it is acceptable to symlink that checkout's `node_modules` into the worktree for validation runs.
- Git commands that mutate repository state must never be run in parallel within the same repository or worktree.
- Do not use parallel tool execution for `git add`, `git commit`, `git rebase`, `git merge`, `git cherry-pick`, `git worktree add`, `git worktree remove`, `git stash`, or any other git command that writes refs, the index, or worktree metadata.
- Do not use `multi_tool_use.parallel` for git commands unless every git command in that batch is strictly read-only.
- Serialize all git write operations and wait for each one to finish before starting the next.
- Treat `.git/index`, `.git/ORIG_HEAD`, `.git/refs/*`, and `.git/worktrees/*` as shared lock-producing paths.
- Never run `git add` and `git commit` in parallel.
- Never run `git status` in parallel with git write operations in the same worktree.
- Commit and validate the change inside the worktree first. For large tasks, milestone commits may accumulate in the worktree, but the worktree is not merge-ready until the full planned workset is complete.
- When the worktree change is ready to land, inspect the shared `main` checkout first. If `main` has any uncommitted tracked changes, stop and tell the user instead of merging.
- Before attempting to merge back, rebase the worktree branch onto the current `main` so parallel agent commits already landed on `main` are incorporated first.
- If the rebase hits conflicts, stop there and tell the user what conflicted so they can decide the next step.
- After a clean rebase, rerun `npm run build` and `npm test` in the rebased worktree. If either fails, stop and tell the user instead of merging.
- Only after `main` is clean, the rebase is conflict-free, the rebased worktree passes build and tests, and the full requested workset is complete should the agent merge the branch back into `main`.
- Merging back into the shared `main` checkout can require sandbox approval because git updates shared refs such as `ORIG_HEAD`; request escalation when the environment blocks that write.
- Do not assume a fast-forward merge will be available before rebasing; parallel agent work commonly means the landing branch has diverged from `main`.
- After merging back into `main`, rerun `npm run build` and `npm test` on `main`, and only then report the task complete.
- Remove the temporary worktree after the change has been safely integrated unless the user asks to keep it.

## Configuration & Data Notes

Default data path is `vendor/pf2e`. Startup is offline-only: it expects a current local SQLite index and prepared embedding assets. If the PF2E checkout, embedding model, or index schema changes, rerun `npm run refresh-index` or `npm run refresh-external`.
