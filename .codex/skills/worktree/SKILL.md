---
name: worktree
description: Use when the user explicitly asks to use a worktree or when a task has a concrete need for isolated checkout work, such as parallel implementation agents, risky long-running edits, preserving a dirty main checkout, or landing a branch from a task worktree.
---

# Worktree

Use this skill to establish and manage an isolated task checkout. Follow the
user's instructions and AGENTS.md for when isolation is required, branch bases,
commit milestones, validation, PR publication and cleanup timing.

## Choose and Create

Inspect existing task worktrees and reuse the checkout for related feedback.
Confirm the intended workstream/base, current Git state and available disk space.
Preserve unrelated changes and running previews.

Choose a path within the session's writable roots, normally
`<repo-root>/.worktrees/<task-slug>`. State the path and its purpose.
Create the branch from the verified base, serializing Git mutations:

```bash
git worktree add --no-track -b <branch> <repo-root>/.worktrees/<task-slug> <base-ref>
git -C <repo-root>/.worktrees/<task-slug> status --short --branch
```

Do task edits in that checkout. Inspect the scoped diff and run the applicable
checks before local commits according to repo policy. Independent review is a
pre-PR requirement, not a commit requirement; follow the
[development sequence](../../../CONTRIBUTING.md#development-loop) before publication.

## Retention and Cleanup

Retain the task checkout while its PR or local build needs review. Record its
path, purpose, owner and removal condition in the handoff. Reuse one
checkout-local compiler target and apply the repo's build-output retention policy.

After merge or cancellation, inspect tracked, untracked and ignored files,
active processes and incoming dependency links. Preserve needed work and user
state; account for disposable outputs before removal. Remove eligible worktrees
serially with `git worktree remove`, then verify the inventory. Report any retained
checkout and why it still needs to exist.
