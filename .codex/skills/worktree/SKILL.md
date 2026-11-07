---
name: worktree
description: Create or retire an Atlas linked worktree, accounting for Cargo configuration inheritance, checkout-local output and shared development resources.
---

# Atlas worktrees

Read [worktrees and cleanup](../../../docs/development/worktrees-and-cleanup.md)
for the repository's checkout, build-output and resource-safety procedures. Task
instructions determine when isolation and cleanup are authorized.

Inspect existing worktrees, Git status, running previews and available disk before
creating another checkout. Reuse a suitable checkout for related work. Prefer an
external or sibling path: Cargo inherits ancestor `.cargo/config*`, so a nested
checkout can duplicate compiler/linker flags and invalidate retained-tool receipts.
If a nested checkout is necessary, account for its configuration stack explicitly.

Create from the verified base, serializing Git mutations:

```bash
git worktree add --no-track -b <branch> <external-worktree-root>/<task-slug> <base-ref>
git -C <external-worktree-root>/<task-slug> status --short --branch
```

Keep Cargo output checkout-local; compiled fixtures embed checkout paths. Before
retirement, inspect tracked/untracked/ignored files, active processes and incoming
dependency links. Preserve uncommitted source, retained data, referenced shared
baselines and live dependency targets. Resolve symlinks and configured workspace
paths so cleanup cannot reach the installed Atlas workspace. Remove eligible
checkouts serially with `git worktree remove` and verify the resulting inventory.
