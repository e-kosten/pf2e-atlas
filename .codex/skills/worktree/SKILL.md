---
name: worktree
description: Use when the user explicitly asks to use a worktree or when a task has a concrete need for isolated checkout work, such as parallel implementation agents, risky long-running edits, preserving a dirty main checkout, or landing a branch from a task worktree.
---

# Worktree

Use this skill only when the user asks for a worktree or the task has a specific isolation need. Do not use it as the default for every implementation task.

Good reasons to use a worktree:

- parallel agents need to edit tracked files
- the current checkout has unrelated changes that should not mix with the task
- the task is long-running, risky, or likely to pause mid-state
- the user wants work isolated on a branch
- the task needs the repo landing script for a linked worktree

If none of those apply, work in the current checkout and do not create a worktree.

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
- use a deterministic child path such as `<repo-root>/.worktrees/<task-slug>` or the fallback root equivalent
- do not use symlink-resolved alternates such as `/private/tmp/...`
- if the chosen path is not under an allowed root, stop before running `git worktree add`

State the chosen worktree path and the reason for using a worktree before editing tracked files there.

## Creation

Create implementation branches from the shared main checkout, but do tracked edits for the isolated task in the task worktree.

Use serialized git write commands:

```bash
git worktree add -b <branch> <repo-root>/.worktrees/<task-slug> main
```

Then verify:

```bash
git -C <repo-root>/.worktrees/<task-slug> status --short --branch
```

If a command tries to write outside the allowed root or asks for approval, treat that as a root-selection bug first. Do not keep working in the wrong location.

## During Work

- Keep isolated task edits inside the task worktree.
- Do not share a checkout with another running agent.
- Do not run git write commands in parallel.
- Do not run `git status` in parallel with git write commands in the same repo.
- Do not revert unrelated main or worktree changes.
- If a worktree is found in the wrong place, stop, inspect its status, remove it with `git worktree remove --force <path>` only if it contains no needed work, delete the temporary branch if empty, and recreate under the allowed root.

## Commit Gate

Commit only after the user explicitly asks to prepare a commit and the `$prepare-commit` skill has completed its validator loop.

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
- rerun `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo build --workspace` in the rebased worktree
- if any check fails or the rebase conflicts, stop before landing

Land with the repo script from the linked task worktree:

```bash
scripts/land-worktree.sh
```

After landing, rerun the same lint/build/test gate on `main`.

## Cleanup Gate

Before reporting a worktree-backed task done:

- remove every temporary worktree created for the task
- verify `git worktree list` no longer shows stale task worktrees
- report the commit SHA and commit message after a prepare-commit workflow creates one
- note any skipped validation, refresh step, data/index implication, or remaining blocker

Cleanup is part of done for worktree-backed tasks. Do not leave temporary worktrees behind unless the user explicitly asks to keep them.
