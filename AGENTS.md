# Repository agent guidance

PF2e Atlas is a Rust workspace with a React/Ant Design frontend in
`web/atlas-ui`. Start with [CONTRIBUTING](CONTRIBUTING.md) for the development
sequence and essential commands. Detailed owners below apply when their trigger
matches the task; read those sections before changing that behavior.

## Development and delivery

- Complete the agreed scope, integration, tests, documentation and review fixes.
  Ask for unresolved consequential decisions or real blockers.
- Use logical local commits at coherent, validated boundaries. Independent review
  is required before merge readiness, not before each local commit.
- Obtain independent review, fix confirmed findings and complete required local
  validation before marking a PR ready for merge. Draft PRs identify remaining
  implementation, review and validation work.
- UI changes require browser interaction and visual evidence alongside automated
  checks; follow the frontend and validation guides for coverage.
- Full applicable Rust/frontend gates run locally (standalone for developer-only
  tooling); browser checks target affected
  behavior. CI owns the full applicable platform/browser matrix by default.
  Reuse valid checks for unchanged inputs; report the candidate actually tested.
- Keep related review fixes on the same PR. Verify CI on the published candidate
  before reporting it ready to merge. Implementation, local validation, human
  acceptance and hosted CI are separate outcomes.
- Do not merge PRs, push directly to shared integration branches or `main`,
  publish releases, or modify the real installed Atlas workspace without explicit
  authorization. Respect requests such as plan-only, no commits or local-only.

## Read the owner for the task

| Before doing this work | Read |
| --- | --- |
| Module ownership, shared abstractions, service boundaries or dependencies | [Architecture overview](docs/architecture/overview.md), then the relevant [ADR](docs/architecture/decisions/README.md) |
| Runtime, ingest, search or artifact/storage changes | [Runtime](docs/architecture/runtime.md) and [artifact contract](docs/architecture/artifact-contract.md), relevant sections |
| Saved lists, parties, encounters, their children, or derived saved-state reads | [Saved-state ownership](docs/architecture/decisions/0051-persistent-workspace.md#saved-state-ownership) |
| Routes, wire DTOs, serde omission/null/defaults or transport encoding | [Browser/backend contract](docs/architecture/decisions/0029-local-web-app-boundary.md#browserbackend-compatibility); update generated bindings or the explicit protocol revision and behavior tests as applicable |
| Rust modules, crate entry points, refactors, input checks or test seams | [Code and refactor guidance](docs/development/code-guidelines.md) |
| User-visible behavior or copy, including Rust-projected text | [Product presentation](docs/product-guidelines.md) |
| Frontend edits, layout, interactions or browser recovery | [Frontend AGENTS](web/atlas-ui/AGENTS.md) and its linked task-specific frontend guidance |
| Local setup, full-corpus preview or frontend hot reload | [Setup and preview](docs/development/setup-and-preview.md) |
| Selecting checks, final validation or diagnosing CI/browser failures | [Validation](docs/development/validation.md) |
| Creating/retiring a worktree, allocating build output or cleaning task data | [Worktrees and cleanup](docs/development/worktrees-and-cleanup.md) |
| Source declaration research or Rust developer diagnostics | [Source research](docs/development/source-research.md) or [developer commands](docs/development/developer-commands.md) |
| Release preparation/publication | [Release procedure](docs/development/releases.md); publication still requires authorization |
| Creating or reorganizing documentation/instructions | [Documentation ownership](docs/development/documentation.md) and [maintain-project-guidance](.codex/skills/maintain-project-guidance/SKILL.md) |

Do not assume frontend instructions were loaded when starting from the repository
root. Explicitly read the frontend AGENTS for frontend work. The crate map belongs
in the architecture overview; detailed procedures belong in their linked guides.

## Architecture and refactors

Architecture docs describe intended boundaries, not optional background. Read
the relevant owner before changing composition or policy. Update code and
architecture docs together when their intended boundaries change; record durable,
non-obvious decisions in an existing or new ADR.

Keep product semantics in Rust app-service/app-model. The frontend renders
generated contracts and calls the thin API client; it must not infer gameplay
rules from raw source JSON or prose. Ant Design owns generic frontend controls;
Atlas components serve product-specific presentation.

Finish refactors by removing the old path and updating callers. Do not introduce
transitional shims, adapters or compatibility layers unless the user requests an
incremental strategy. Search for leftover old paths before reporting completion.

## Workspaces and resources

Use a task branch and isolate concurrent or unrelated work with a linked worktree
when needed. Confirm the intended base and preserve unrelated changes and active
previews. Serialize Git mutations, including worktree operations, within the
repository.

Protect the real installed Atlas workspace and explicitly retained data. Resolve
symlinks and configured paths before cleanup. Follow the
[cleanup procedure](docs/development/worktrees-and-cleanup.md) to establish resource
ownership and retention needs before removing task data.

Reuse one task preview URL and its saved state. Refresh deliberately; frontend-only
work can use Vite with a compatible running backend. Human previews use the
full-corpus workspace and verified models; automated browser fixtures are separate.
Do not reset preview state or run setup automatically on compatibility failure.
For a stale shared preview baseline, establish compatibility and coordinate a
shared-default update using the linked setup guide.

Reuse one checkout-local Cargo target; do not share targets across worktrees.
Preserve active output and shared source/model caches. Follow the linked resource
policy before expensive builds or additional database copies.

## Plans and completion

For cross-cutting changes, identify the affected owners, acceptance cases and
validation obligations before implementation. Preserve the agreed scope and
explicit non-goals during review. Keep durable design decisions in architecture
docs and temporary task status in task notes. Follow the
[issue tracking guidance](CONTRIBUTING.md#tracking-future-work) when discussing
or filing future work.

Respect tracked hooks. Commit hooks check formatting/whitespace; substantive
validation and independent review are required before merge readiness.
For documentation, follow the [audience and task guidance](docs/development/documentation.md#write-for-the-pages-reader)
and check the reader's complete task path. Prose-only changes need document checks,
not unrelated Rust/UI gates.
Keep local/CI results and any unavailable review evidence explicit.

When retiring task resources or synchronizing a checkout after merge, follow the
cleanup guide's ownership, retention and safe fast-forward checks. Repository
procedures do not grant permission to modify another contributor's resources.
