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

For a reusable development environment with the full reference content, use
`just dev-workspace-init` and `just dev-status`. These commands use a shared,
prebuilt backend by default, which may differ from the code in your checkout.
Read [setup and previews](docs/development/setup-and-preview.md) before choosing
a backend. Keep Cargo output checkout-local and reuse existing builds.

## Development loop

Complete the agreed scope, integration, tests, documentation and review fixes.
Use this sequence:

1. For cross-cutting source-derived rules or wire changes, record the affected
   producers/consumers and focused domain/transport cases in the existing task
   plan before implementation; see [early validation](docs/development/validation.md#early-domain-and-transport-checks).
   Implement with focused checks and logical local commits. Each commit should
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

Use focused checks during iteration and the applicable full gates before opening
a PR. The [validation guide](docs/development/validation.md) owns gate selection,
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

Use one concise local plan for substantial work; ordinary fixes need none.
The [repo-plan skill](.codex/skills/repo-plan/SKILL.md) owns plan placement,
status, navigation and archival. Plans are local working artifacts; durable
boundaries and remaining work belong in architecture and backlog docs. Do not
commit local plans unless requested; plan approval is not execution authorization.

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
