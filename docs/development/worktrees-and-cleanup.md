# Worktrees, build output and cleanup

Read this guide before creating or retiring a task checkout, allocating a large
preview, changing build profiles, or reclaiming development data.

## Worktree lifecycle

Use a linked worktree when implementation needs an isolated checkout:

- create it outside the primary checkout directory, on a short-lived feature branch
- branch from current `main` unless the user identifies another workstream; dependent PRs may target the preceding feature branch
- source-modeling work uses `integration/source-modeling` when that is the identified workstream; verify the base and do not mix unrelated workstreams
- continue related feedback and revisions in the same task worktree and PR
- preserve unrelated changes and running builds being reviewed
- land through a reviewed PR after applicable validation and CI; merging is a separate human decision

Cargo reads ancestor `.cargo/config*` files and merges array values, including
compiler/linker flags. A nested `.worktrees/<task-slug>` can therefore build with
both its own and the primary checkout's configuration. Prefer an external managed
worktree or a sibling location for new tasks; keep the private `target/` there.
Do not relocate an active reviewed checkout merely to apply this preference.
Retained-tool readiness fingerprints actual ancestor configuration rather than
silently treating different compiler inputs as equivalent. If a session can only
write a nested location, account for that stack and its explicit refresh requirement.
See [Cargo configuration hierarchy](https://doc.rust-lang.org/cargo/reference/config.html#hierarchical-structure).

Recommended branch names:

- `feat/<topic>`
- `fix/<topic>`
- `docs/<topic>`
- `chore/<topic>`

Follow the [contribution sequence](../../CONTRIBUTING.md#development-loop) for local commits, independent review and PR publication. Establish resource ownership and task authorization before cleanup; this procedure supplies no standing permission to remove data, merge or change the installed workspace.

Keep the worktree available while its PR or local build is being reviewed. When retiring it, stop only its task-owned previews and preserve needed uncommitted source work. Check for another active contributor, build or dependency before removal. Retaining a branch does not require retaining its checkout or compiler cache.

Before removal, inspect Git status, untracked and ignored files, running previews/builds, and incoming symlinks. Preserve needed uncommitted source files and retained data; identify which development outputs can be discarded. A live `node_modules` symlink target must remain even if its original PR has merged. Remove worktrees with `git worktree remove`, one at a time, and verify the inventory. Never force-remove needed source work or another checkout's dependency. Use `--force` only after accounting for every file it would discard; do not broadly delete `.worktrees/`.

Read-only exploration may use the current checkout. Worktree creation/removal,
branch changes and commits must be serialized within the repository. Task
instructions determine authorization and retention timing.

For retained worktrees, identify their purpose and any active process or dependency. A stable preview normally does not require a compiler cache. Retire temporary review checkouts when their review and retention needs have ended.

## Build Profiles And Disk Budget

Ordinary development and tests use limited debug information and enable
incremental compilation. This keeps source/function backtraces and qualified
function names and reuses compiler work during editing. The large generated
`atlas-foundry-model` crate disables incremental compilation to avoid retaining
its extra compiler cache; Cargo still reuses its unchanged completed output.
Clean CI jobs set `CARGO_INCREMENTAL=0` globally. Tests inherit development settings,
including debug assertions and overflow checks; release uses its own profile.

For a debugger session that needs variable/type inspection, use the full-debug
profile for both builds and tests:

```bash
cargo build -p atlas-cli --profile debug-full
cargo test -p atlas-local-state --profile debug-full
```

That profile writes to `target/debug-full`. Retain it while debugging; remove it
when the session is over and the output is no longer needed:

```bash
cargo clean --profile debug-full
```

Use the checked-in defaults consistently across manual checks and hooks. Avoid
profile/env changes for individual agents or validation retries, and retire
obsolete configurations rather than retaining both indefinitely. Normal
`git push` needs no override. Keep active targets: routinely deleting and
rebuilding them adds latency and disk writes. Neither retained size nor build
time alone establishes SSD write volume or endurance.

Before creating a worktree, copying a preview workspace or running expensive gates,
check free space and the current task's compiler and runtime output:

```bash
df -h .
git worktree list
test ! -d target || du -sh target
test ! -d .atlas || du -sh .atlas
```

Below 30 GiB free, reclaim obsolete task build output before starting another
large build. A target above 15 GiB warrants inspecting `debug/deps`,
`debug/incremental`, and `debug/build` for accumulated configurations. These
thresholds prompt investigation, never automatic deletion of active files.
Monitor growth during long builds. Preserve shared source/model assets and
artifacts while an active task or review needs them. Shared baseline versions and
referenced source/model assets must survive task worktree retirement; keep a
baseline while a workspace receipt or running process references it.

Keep one checkout-local Cargo target per task; do not copy or symlink Cargo
outputs across checkout paths. Before deleting output, check for active builds,
reviews and dependency links. Preserve active output and live dependency targets
such as `node_modules` while another checkout uses them. For cleanup work, report
what was removed or retained and the measured space recovered.

## Development Data And Task Cleanup

Establish ownership, retention needs and authorization before deleting development
data. Repository-local state can include preview databases, saved lists and
encounters edited during review, migration backups, fixture exports and copied
reference corpora. Its location alone does not make it disposable. Preserve
uncommitted source work, another contributor's active resources and retained data.

The user's real installed Atlas workspace is protected. Resolve symlinks and
configured paths before deleting data: a path under the repo can point to the real
installation, and a command using global path mode can select a development
workspace through an explicit override. Never reset, replace or delete the real
installed workspace without explicit authorization. Normal Atlas setup continues
to preserve saved state; development cleanup does not change product behavior.

Reuse the task workspace and stable URL during review under
[Local Build Review](setup-and-preview.md#local-build-review). Before another full
workspace copy, inspect existing databases, upgrade backups and task-owned
preview processes. Superseded copies can be retired after checking recovery and
retention needs. Preserve the shared baseline while active checkouts reference it.

To synchronize a clean primary checkout after an authorized merge, use
`git fetch origin` and `git merge --ff-only origin/main`, then verify its HEAD
matches the fetched remote. Preserve local changes or divergence and report any
blocked update; do not reset or stash work to force cleanup. This procedure does
not authorize modifying someone else's checkout, merging a PR or pushing to `main`.

At a schema/reference-contract or developer-capability milestone, also inspect
the shared tool/seed readiness. Refresh explicitly from a validated clean build
with the [shared refresh procedure](setup-and-preview.md#reusable-development-baseline).
Updating Git alone does not update retained executables or seeds. Ordinary UI or
prose changes do not require republication, and existing task receipts stay pinned.
