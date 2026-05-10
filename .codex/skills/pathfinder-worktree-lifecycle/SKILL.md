---
name: pathfinder-worktree-lifecycle
description: Use for Pathfinder MCP implementation work that creates, validates, commits, lands, or cleans up git worktrees; prevents wrong-root worktrees, premature merges, missing lint/build/test validation, unsafe git writes, and approval prompts caused by paths outside the active writable roots.
---

# Pathfinder Worktree Lifecycle

Use this skill before any tracked implementation edit in this repository, and again before committing, landing, or cleaning up a task worktree.

## Root Selection

Choose the worktree root from the current session's writable roots, not from generic prose.

Default for this repo:

```text
<repo-root>/.worktrees
```

Before creating a worktree:

- inspect the active permissions instructions for writable roots
- prefer `<repo-root>/.worktrees` when writable
- if that path is unavailable, use an allowed writable root from environment instructions with the same deterministic shape
- use a deterministic child path such as `<repo-root>/.worktrees/<task-slug>` (or the fallback root equivalent)
- do not use symlink-resolved alternates such as `/private/tmp/...`
- if the chosen path is not under an allowed root, stop before running `git worktree add`

State the chosen worktree path in a short update before editing tracked files.

## Creation

Create implementation branches from the shared main checkout, but do all tracked edits in the task worktree.

Use serialized git write commands:

```bash
git worktree add -b <branch> <repo-root>/.worktrees/<task-slug> main
```

Then verify:

```bash
git -C <repo-root>/.worktrees/<task-slug> status --short --branch
```

If `package.json` and `package-lock.json` match the main checkout, symlink dependencies inside the allowed worktree root:

```bash
ln -s /path/to/repo/node_modules <repo-root>/.worktrees/<task-slug>/node_modules
```

If a command tries to write outside the allowed root or asks for approval, treat that as a root-selection bug first. Do not keep working in the wrong location.

## During Work

- Keep implementation edits inside the task worktree.
- Keep planning-only untracked files under `scratch/plans/` in main only when the user requested planning.
- Do not run git write commands in parallel.
- Do not run `git status` in parallel with git write commands in the same repo.
- Do not revert unrelated main or worktree changes.
- If a worktree is found in the wrong place, stop, inspect its status, remove it with `git worktree remove --force <path>` only if it contains no needed work, delete the temporary branch if empty, and recreate under the allowed root.

## Validation Gate

Before saying implementation is complete, run the validation required by the task plus the repo landing gate.

For implementation work, the minimum gate is:

```bash
cd scripts && npm run lint
npm run build
cd scripts && npm test
```

For plan-driven work, also validate directly against the plan file:

- every slice or checklist item is complete
- docs and ADR follow-through are complete
- no plan item is deferred, partial, or hidden as follow-up
- refactors have no compatibility shims, duplicate paths, or mixed old/new ownership left behind
- any required grep/code-audit checks have been run and recorded

Do not commit a failing implementation unless the user explicitly asks for that state.

## Commit Gate

Before committing:

- inspect `git status --short`
- stage only files for the completed task
- commit inside the task worktree with a Conventional Commit message
- include validation evidence in the final user report

Do not commit unrelated dirty files. If unrelated changes block a clean commit boundary, stop and explain the blocker.

## Landing Gate

Do not merge or land a task worktree until the full requested workset is complete.

Before landing:

- inspect the shared main checkout
- if main has uncommitted tracked changes, stop and report that landing is blocked
- rebase the worktree branch onto current `main`
- rerun `cd scripts && npm run lint`, `npm run build`, and `cd scripts && npm test` in the rebased worktree
- if any check fails or the rebase conflicts, stop before landing

Land with the repo script from the linked task worktree:

```bash
scripts/land-worktree.sh
```

After landing, rerun the same lint/build/test gate on `main`.

## Cleanup Gate

Before reporting done:

- remove every temporary worktree created for the task
- verify `git worktree list` no longer shows stale task worktrees
- report the commit SHA and commit message for completed implementation work
- note any skipped validation, refresh step, data/index implication, or remaining blocker

Cleanup is part of done. Do not leave temporary worktrees behind unless the user explicitly asks to keep them.
