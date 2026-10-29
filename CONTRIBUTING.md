# Contributing

PF2e Atlas is a Rust workspace with a React frontend. This is the contributor
entry point; use the task-specific guides below for detailed procedures. Product
setup and everyday use belong in [README.md](README.md).

## Quick start

Implementation uses a dedicated linked worktree on a short-lived feature branch.
Reuse the task checkout for related revisions, preserve unrelated changes and
active previews, and verify the intended base before editing. Start from current
`main` unless the task identifies another workstream or a dependent PR.

From the task checkout, install the tracked hooks and verify the checkout:

```bash
scripts/install-git-hooks.sh
scripts/preflight.sh
# or: just dev-setup
```

For an existing full-corpus development baseline, use `just dev-workspace-init`
and `just dev-status`. Read [setup and previews](docs/development/setup-and-preview.md)
before selecting a backend: these commands default to a frozen baseline binary,
not the current checkout. Keep Cargo output checkout-local and reuse warm builds.

## Development loop

Complete the agreed scope, integration, tests, documentation and review fixes.
Use this sequence:

1. Implement with focused checks and logical local commits. Each commit should
   express a coherent, validated change and leave the branch buildable.
2. For substantive UI work, share an early usable local preview and get user
   feedback before broad test rewrites and final validation. Internal refactors,
   backend-only work and minimal UI changes can skip this checkpoint or provide
   a later preview. Preserve existing tests and disclose unfinished behavior.
3. Finish the feature locally, then obtain independent subagent review against
   the agreed scope, relevant architecture and behavior. Give reviewers the
   candidate, acceptance criteria and validation evidence. Scale reviewer count
   and specialties to the change. UI work includes actual browser interaction
   and visual review by Astra subagents alongside automated checks.
4. Resolve confirmed findings and material evidence gaps; obtain focused rechecks
   where needed. Independent review is a pre-PR gate, not a commit gate.
5. Run the required final local checks on the resulting candidate. Reuse passing
   evidence when code, inputs and toolchain are unchanged; rerun affected checks
   after substantive changes.
6. Open a PR only after implementation, independent review fixes and required
   local validation are complete. Early drafts require an explicit user request.
7. Complete CI on the published head before merge readiness. Local validation,
   CI and human acceptance are separate outcomes; report each truthfully.

Keep a preview at one stable task URL and preserve review edits across refreshes.
Reuse a compatible backend and Vite hot reload for frontend-only iterations.
See [local build review](docs/development/setup-and-preview.md#local-build-review)
for the feedback and refresh lifecycle. Ask for input on unresolved consequential
product/architecture decisions or real blockers, rather than routine commits.

## Validation

During iteration, run focused tests. Before opening the PR, run comprehensive
checks for affected code: the full Rust gate for application/shared Rust changes,
the standalone developer gate for developer-only changes, the frontend gate
for UI changes, and relevant integration/browser scenarios. CI owns the full
applicable platform and device/browser matrix by default. Broaden local browser
coverage for shared layout, navigation, overlays, touch/focus or harness changes.

```bash
just verify-changed --range origin/main...HEAD --list
just verify-changed --range origin/main...HEAD --full
```

Substitute the actual PR base for stacked or integration work. Install required
dependencies explicitly; gates do not install them. Prose-only changes need
accuracy, whitespace, links and example checks, without unrelated application
builds. Read [validation](docs/development/validation.md) for commands, candidate
consistency, test selection, CI diagnosis, visual references and hooks.

## Commit messages and publication

Use Conventional Commits, for example `fix(encounters): preserve source conditions`.
An optional description body must follow a blank line:

```text
type(scope): summary
```

```text
type(scope): summary

description
```

Recommended types are
`feat`, `fix`, `docs`, `refactor`, `test` and `chore`. A PR can contain several
logical commits; commit boundaries do not require separate PRs or user approvals.
Do not combine unrelated changes or accumulate an entire multi-stage task uncommitted.

Authorization to implement includes logical local commits, pushing the task branch
and opening/updating its PR after the pre-PR requirements above. Planning alone
does not authorize implementation. Explicit plan-only, no-commit or local-only
instructions take precedence. Keep related review fixes on the same PR as
additional logical commits; agree before rewriting published history.

Describe the final behavior, validation evidence and remaining limitations in the
PR. Do not merge, push directly to shared integration branches or `main`, publish
releases, or modify the real installed Atlas workspace without explicit permission.
Passing checks does not provide merge authorization.

## Working plans

Use one concise local plan for substantial work under `scratch/plans/`; ordinary
fixes need no new plan. Refine the same plan as scope evolves, record status and
acceptance checks, and update `scratch/plans/README.md` when present. Archive
completed/superseded plans while retaining provenance-bound originals. Plans are
local working artifacts; architecture and backlog remain durable shared guidance.
Plan approval and execution authorization are separate; do not commit local plans
unless explicitly requested.

## Task-specific guidance

Read the applicable guide when the task reaches its scope:

| Task | Maintained guide |
| --- | --- |
| Prepare a checkout, select a backend, publish or refresh a preview | [Setup and previews](docs/development/setup-and-preview.md) |
| Select checks, validate a candidate, diagnose CI/browser failures | [Validation](docs/development/validation.md) |
| Create/retire a worktree, inspect disk growth, clean development data | [Worktrees and cleanup](docs/development/worktrees-and-cleanup.md) |
| Run source/artifact developer diagnostics or measure processing | [Developer commands](docs/development/developer-commands.md) |
| Change source pins/generated models or run corpus/embedding probes | [Source research](docs/development/source-research.md) |
| Prepare nightlies, releases, installers or packaging | [Releases](docs/development/releases.md) |
| Change documentation or contributor/agent instructions | [Documentation ownership](docs/development/documentation.md) |
| Change module structure or perform a refactor | [Code guidelines](docs/development/code-guidelines.md) and [architecture overview](docs/architecture/overview.md) |
| Change UI controls or projected user copy, including Rust text | [Product guidelines](docs/product-guidelines.md) and [frontend guide](web/atlas-ui/docs/frontend-guidelines.md) |

The [architecture overview](docs/architecture/overview.md) owns crate navigation;
read relevant architecture documents and ADRs before changing ownership,
composition, dependencies, shared abstractions, storage or retrieval boundaries.
Keep those documents aligned with the implementation in the same task.
