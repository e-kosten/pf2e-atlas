# Contributing

PF2e Atlas is a Rust workspace with a React frontend. This is the contributor
entry point; use the task-specific guides below for detailed procedures. Product
setup and everyday use belong in [README.md](README.md).

## Quick start

Use a short-lived feature branch. A linked worktree is useful for concurrent
changes or preserving an existing checkout; see [worktrees and cleanup](docs/development/worktrees-and-cleanup.md).
Reuse a suitable checkout for related revisions, preserve unrelated changes and
active previews, and verify the intended base before editing. Start from current
`main` unless the task identifies a dependent PR or another explicit base.

From the task checkout, install the tracked hooks and verify the checkout:

```bash
scripts/install-git-hooks.sh
scripts/preflight.sh
# or: just dev-setup
```

For a reusable development environment with the full reference content, use
`just dev-workspace-init` and `just dev-status`. These commands use a shared,
prebuilt backend by default, which may differ from the code in your checkout.
Read [setup and previews](docs/development/setup-and-preview.md) before choosing
a backend. Keep Cargo output checkout-local and reuse existing builds.

## Development loop

Complete the agreed scope, integration, tests, documentation and review fixes.
Use this sequence:

1. For cross-cutting source-derived rules or wire changes, record the affected
   producers/consumers and focused domain/transport cases before implementation;
   see [early validation](docs/development/validation.md#early-domain-and-transport-checks).
   Implement with focused checks and logical local commits. Each commit should
   express a coherent, validated change and leave the branch buildable.
2. For UI changes, use a functional preview to review the affected browser flows
   and presentation. Preserve existing tests and disclose unfinished behavior.
   See [local build review](docs/development/setup-and-preview.md#local-build-review).
3. Obtain independent review against the agreed scope, relevant architecture and
   behavior. Give reviewers the candidate, acceptance criteria and validation
   evidence. Scale review depth to the change. UI work includes actual browser
   interaction and visual review alongside automated checks.
4. Resolve confirmed findings and material evidence gaps; obtain focused rechecks
   where needed. Independent review is required before merge readiness, not before
   each local commit.
5. Run the required final local checks on the resulting candidate. Reuse passing
   evidence when code, inputs and toolchain are unchanged; rerun affected checks
   after substantive changes.
6. Mark a PR ready for merge after implementation, independent review fixes and
   required local validation are complete. Draft PRs identify remaining work.
7. Complete CI on the published head before merge readiness. Local validation,
   CI and human acceptance are separate outcomes; report each truthfully.

Keep a preview at one stable task URL and preserve review edits across refreshes.
Reuse a compatible backend and Vite hot reload for frontend-only iterations.
See [local build review](docs/development/setup-and-preview.md#local-build-review)
for the feedback and refresh lifecycle. Resolve consequential product and
architecture questions before implementing behavior that depends on them.

## Validation

Use focused checks during iteration and the applicable full gates before marking
a PR ready for merge. The [validation guide](docs/development/validation.md) owns gate selection,
local/CI coverage, candidate consistency, browser checks and hooks. Prose-only
changes use the [documentation checks](docs/development/documentation.md#validate-the-edited-documentation).

```bash
just verify-changed --range origin/main...HEAD --list
just verify-changed --range origin/main...HEAD --full
```

Substitute the actual PR base for stacked or integration work. Install required
dependencies explicitly; gates do not install them.

## Commit messages and publication

Use Conventional Commits: `type(scope): summary`, for example
`docs(setup): simplify first-run instructions`. An optional body follows a blank
line. Common types are `feat`, `fix`, `docs`, `refactor`, `test` and `chore`.
Keep unrelated changes separate; one PR can contain several logical commits.

Keep related review fixes on the same PR as additional logical commits; coordinate
before rewriting published history. Agent-assisted work follows the task's
explicit authorization for implementation, commits and publication; these
procedures do not grant standing permissions.

Describe the final behavior, validation evidence and remaining limitations in the
PR. Do not merge, push directly to shared integration branches or `main`, publish
releases, or modify the real installed Atlas workspace without explicit permission.
Passing checks does not provide merge authorization.

## Working plans

For substantial changes, make scope, affected owners, dependencies and acceptance
checks clear enough for implementation and review. Task notes and planning tools
may vary between contributors. Keep durable boundaries in architecture documents
and remaining work in the backlog; do not turn temporary execution notes into
repository policy.

## Documentation changes

Write for the page's reader and the task they came to complete. Follow the
[documentation guide](docs/development/documentation.md#write-for-the-pages-reader)
for audience, example and reference boundaries, and check the full task path as
well as individual sentences. Keep detailed maintenance policy in that guide.

## Task-specific guidance

Read the applicable guide when the task reaches its scope:

| Task | Maintained guide |
| --- | --- |
| Prepare a checkout, select a backend, publish or refresh a preview | [Setup and previews](docs/development/setup-and-preview.md) |
| Select checks, validate a candidate, diagnose CI/browser failures | [Validation](docs/development/validation.md) |
| Update dependencies, triage advisories or review embedding compatibility | [Maintaining dependencies](docs/development/dependencies.md) |
| Create/retire a worktree, inspect disk growth, clean development data | [Worktrees and cleanup](docs/development/worktrees-and-cleanup.md) |
| Run source/artifact developer diagnostics or measure processing | [Developer commands](docs/development/developer-commands.md) |
| Change source pins/generated models or run corpus/embedding probes | [Source research](docs/development/source-research.md) |
| Prepare nightlies, releases, installers or packaging | [Releases](docs/development/releases.md) |
| Change documentation or contributor/agent instructions | [Documentation ownership](docs/development/documentation.md) |
| Change module structure or perform a refactor | [Code guidelines](docs/development/code-guidelines.md) and [architecture overview](docs/architecture/overview.md) |
| Change saved lists, parties, encounters, or reads derived from them | [Saved-state ownership](docs/architecture/decisions/0051-persistent-workspace.md#saved-state-ownership) |
| Change UI controls or projected user copy, including Rust text | [Product guidelines](docs/product-guidelines.md) and [frontend guide](web/atlas-ui/docs/frontend-guidelines.md) |
| Add, move or change a CLI command | [CLI command design](docs/product-guidelines.md#cli-command-design) |

The [architecture overview](docs/architecture/overview.md) owns crate navigation;
read relevant architecture documents and ADRs before changing ownership,
composition, dependencies, shared abstractions, storage or retrieval boundaries.
Keep those documents aligned with the implementation in the same task.
