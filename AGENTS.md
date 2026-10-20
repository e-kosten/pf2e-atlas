# Repository Guidelines

## Project Structure & Module Organization

PF2e Atlas is a Rust workspace. Core application code lives under `crates/`.

- `crates/atlas-cli`: product command parsing, presentation, exit codes, and agent skill installation.
- `crates/atlas-dev`: local Rust developer commands for source analysis, path auditing, and artifact inspection; excluded from releases.
- `crates/atlas-cli-support`: shared path/progress argument vocabulary, JSON envelopes, and progress rendering for both Rust CLIs.
- `crates/atlas-runtime`: paths, setup/source-fetch policy, requested configuration, shared migration composition, and bounded typed workspace operations.
- `crates/atlas-storage`: connection admission, bounded sessions, maintenance ownership, migration execution, and backups; no domain schema ownership.
- `crates/atlas-search`: product-facing retrieval orchestration.
- `crates/atlas-index`: reference schema, incremental publication, embedding checkpoints, validation, row readers, filter discovery, filter compilation, and vector SQL.
- `crates/atlas-foundry-model`: generated authored DTOs, ordered source primitives, strict parsing/admission, pure document dispatch and typed snapshot codec.
- `crates/atlas-ingest`: typed Foundry loading, content/reference preparation, attributed indexing inputs, embedding execution and artifact-build composition.
- `crates/atlas-embedding`: the two pinned search-profile contracts, shared baseline segmentation, native budgets, retained query sessions and FastEmbed inference.
- `crates/atlas-record`: source-backed records, borrowed authored views, selected content, typed query facts and reference policy.
- `crates/atlas-domain`: shared typed query, source address, lightweight summary and canonical record-key vocabulary.
- `crates/atlas-app-service`: product workflows, local overlays and consumer presentation projections.
- `crates/atlas-app-model`: generated frontend contracts for workflows and presentation.
- `crates/atlas-local-state`: saved-domain schema, lists, encounters and versioned receipts on caller-owned workspace sessions.
- `crates/atlas-web`: web transport and embedded frontend hosting.
- `crates/atlas-sqlite-vec`: sqlite-vec registration and capability probing.

The first-party local-agent skill lives in `skills/pf2e-atlas-cli`. Default reference data is acquired at the model pin under `.cache/pf2e/<commit>` in repo mode. A separate `vendor/pf2e` checkout may be used for explicit developer diagnostics.

Private TypeScript contributor packages live under `dev-tools/`. Keep package
configuration at each package root, implementation in `src/`, command parsing and
exit behavior in `src/cli/`, and tests in `tests/`. Source-contract fixtures stay
in the package; large extracted declaration graphs belong in ignored caches and
are regenerated from the pinned upstream source. Operational shell and
PowerShell workflows and installers remain under `scripts/`.

## Architecture Docs

The architecture documents under `docs/architecture/` are part of the working source of truth for this repository, not optional reference material.

- Before making changes that affect module ownership, runtime composition, cross-crate dependencies, shared abstractions, service boundaries, search flow, artifact schema, or future TUI/tagging architecture, read the relevant architecture docs first.
- Start with `docs/architecture/overview.md`. For runtime, ingest, search, or artifact work, also read `docs/architecture/runtime.md` and `docs/architecture/artifact-contract.md`. Read the ADR index in `docs/architecture/decisions/README.md` when the change depends on durable architecture decisions.
- When a change materially alters the architecture or the intended editing guidance, update the relevant docs in the same task so they stay in sync with the code.
- Architecture-impacting changes include new or replaced facades, moved ownership between crates, new lint-enforced boundaries, major search-pipeline changes, storage-boundary changes, and future TUI or editorial workflow restructuring.
- During architectural rework, prefer direct replacement to compatibility layers, adapters, or shims. Treat shim-based transitions as tech debt by default.
- If you add a new durable architectural rule or make a non-obvious architectural choice that future editors will need to preserve, update an existing ADR or add a new file under `docs/architecture/decisions/`.
- Do not report an architecture-impacting implementation task as complete if the code and the architecture docs disagree about the intended structure.

## Documentation Positioning

Keep each documentation surface focused on its job:

- `README.md` should describe the current product, what users can do with it, and how to set it up and run it.
- `docs/architecture/` should describe the current intended architecture and durable boundaries that the codebase is expected to follow.
- `docs/backlog/backlog.md` should list only open backlog work using the active status vocabulary in that file.
- `docs/backlog/history/` should hold durable history of completed or retired backlog work.
- `CONTRIBUTING.md` should carry contributor workflow, validation commands, repo layout, and developer-facing operational guidance.
- Historical context, design evolution, tradeoff records, and "why we changed this" material should primarily live in ADRs under `docs/architecture/decisions/`.

When writing or editing docs, prefer statements of current behavior and current ownership over repo-history framing unless the document is explicitly an ADR or backlog/history note.

## Planning Documents

Use a written plan for substantial work, with one current plan per task. Reuse it
as the discussion evolves; routine fixes and next-step questions need no new file.
Keep plans concise and their status explicit. Consult the local
`scratch/plans/README.md` when present; keep it current and archive completed or
superseded plans with references/provenance preserved. Use the task's appropriate
checkout and base. See the repo-plan skill for the local lifecycle.

## Frontend UI Direction

For `web/atlas-ui`, Ant Design is the selected component library for generic application UI. Prefer Ant components and component tokens for buttons, icon buttons, links, navigation controls, tables, forms, overlays, selectors, alerts, empty/loading states, tags, pagination, and selected/active/hover styling before adding custom controls or local CSS state styling. Custom Atlas components should be reserved for product-specific surfaces such as record presentation, PF2e stat/rendering layouts, rich result summaries that Ant list/table primitives cannot express cleanly, and encounter-runtime panels. See `web/atlas-ui/docs/frontend-guidelines.md` and `web/atlas-ui/AGENTS.md` for concrete frontend editing rules.

For updates to an already displayed document, preserve content, layout, scroll and
focus while fetching the replacement. Delay loading feedback for fast operations;
slow-operation feedback must not shift the content. Keep failures local with Retry
and clearly distinguish retained content from a successful update. Never reuse
content across different source identities or invalidated generations. See the
frontend guidelines for the shared reading implementation.

## Product UI Controls

- Default to a direct action, then inline or anchored context for small frequent
  tasks. Reserve modals for substantial editing or decisions that require focus.
  Keep the pointer, keyboard focus and reading position near the initiating control.
- Rules reminders support play; they must not become mandatory completion checklists.
  Accept outcomes resolved at the table without demanding a raw die result or
  creating fictitious roll history. Optional rolls use the shared dice surface.
- Keep condition identity and essential mechanics visible. Use disclosures only
  for genuinely secondary details such as provenance or additional sources.
- Show predictable consequences and supported opt-outs before the first submission.
  One task has one submission; moving a second step into the same modal is not a
  single-step flow. Compute previews while editing rather than after clicking Apply.
- Prevent invalid input at the control. Disabled actions need an understandable,
  actionable reason; never strand the user behind an unexplained disabled button.
- Adjacent buttons in the same action group must use the same size and aligned
  height. Express primary/secondary importance with variant or emphasis, not a
  smaller secondary button. Dense inline actions may use small buttons only when
  the whole group uses that size. Align adjacent inputs and buttons consistently.
- Keep encounter workflows visually stable. Do not use expanding disclosures as
  the default container for unrelated controls or introduce animated layout shifts
  around frequently used actions. Place contextual information beside its action;
  choose an overlay only for a focused task that needs one.

## Product UI Copy

These rules apply to all user-visible copy, including text projected by Rust into
web and CLI responses.

- Prefer icons for familiar compact actions when their meaning is clear in context,
  with accessible names and concise hover/focus tooltips. This is a contextual
  preference, not an icons-only rule: use text when it improves comprehension,
  distinguishes choices, or makes an unfamiliar action clear.
- Prefer no helper text when the label, value, control or visible state already
  communicates the behavior. Do not add a sentence merely to explain the feature.
- Every note must give a concrete fact, consequence or useful next action. Avoid
  vague phrases such as “needs manual handling,” “resolve explicitly,” or
  “unsupported automation.” State the actual restriction or action when known;
  do not invent one when the source does not establish it.
- Do not narrate implementation boundaries or add blanket automation disclaimers.
  Surface a limitation only where it changes how the user should interpret a
  displayed value or perform the current task.
- Keep essential status beside the affected fact or control. Put useful optional
  calculation details in its existing explanation popover. Do not create a generic
  notes section or move redundant prose into tooltips merely to hide it.
- Review the literal rendered copy before sharing a preview. Delete repetition
  first, then use short, direct wording. Short but vague copy is not an improvement.

For example, omit “Action loss is shown in the action budget; spending actions is
tracked at the table.” The displayed forecast and its explanation already cover
that distinction. Omit “The additional action's permitted uses need manual
handling.” Show a specific restriction such as “Stride or Strike only” only when
supported by the granting effect.

## Build, Test, And Development Commands

- `scripts/install-git-hooks.sh`: configure this clone to use the tracked git hooks in `.githooks/`.
- `scripts/preflight.sh`: verify the current checkout is a linked worktree on a non-`main` branch.
- `just verify` or `scripts/verify.sh`: run the Rust fmt, clippy, test, and build gate.
- `cargo run -p atlas-cli -- --help`: run the product CLI from source.
- `cargo run -p atlas-dev -- --help`: run Rust developer diagnostics without building the web UI or starting Node.
- `cargo run -p atlas-cli -- setup --check --json`: inspect setup readiness from source.
- `cargo install --path crates/atlas-cli --locked`: install the local `atlas` CLI.

## Coding Style & Naming Conventions

This codebase is Rust. Keep modules single-purpose and put helpers beside the concern that owns their policy. Avoid generic `utils.rs` or `helpers.rs` modules. Promote shared helpers only when at least two existing owners need the same stable primitive and there is no clearer domain owner.

Prefer descriptive module names and explicit ownership:

- reference schema, validation, checkpoints and query SQL belong in `atlas-index`
- saved-domain schema and atomic domain actions belong in `atlas-local-state`
- runtime path/setup policy and the composed migration ledger belong in `atlas-runtime`
- connection/session admission, migration execution and backups belong in `atlas-storage`
- retrieval orchestration belongs in `atlas-search`
- typed source loading, preparation and artifact-build composition belong in `atlas-ingest`
- source-backed record/content policies and borrowed authored views belong in `atlas-record`
- consumer presentation and local gameplay overlays belong in `atlas-app-service`
- Product command presentation and exit behavior belong in `atlas-cli`; developer command presentation belongs in `atlas-dev`
- Shared CLI argument vocabulary, JSON envelopes, and progress rendering belong in `atlas-cli-support`; runtime path policy stays in `atlas-runtime`

Use `cargo fmt` for formatting. Keep Clippy clean under the workspace gate.

Keep crate entry points focused on composition:

- `lib.rs` should declare modules, expose the crate's intentional public API with explicit re-exports, and carry only small crate-level glue that has no clearer module owner. Put public model definitions, parsing, projection, storage, and behavior logic in named modules such as `model.rs`, `reader.rs`, `presentation.rs`, or domain-specific submodules, then re-export the public surface from `lib.rs`.
- `main.rs` should wire CLI/runtime entry behavior and delegate command logic to modules. Avoid accumulating command implementations, business logic, or reusable helpers in `main.rs`.
- When a crate root grows large, inspect whether it is large because of public API exports or because it owns real definitions/logic. Prefer moving definitions and logic to cohesive modules while preserving a clear public API at the root.

## Testing Guidelines

Rust workspace tests are the primary validation surface. Add or update tests under the owning crate for behavior changes, especially around ingest, indexing, lookup, search, filter discovery, graph context, and artifact validation.

Before each logical commit, inspect the scoped diff and run checks appropriate to the change. Before marking a PR ready for review, run the full applicable gates: the Rust gate for Rust changes and the frontend gate for web UI changes. Respect the tracked hooks; do not bypass them. Reuse successful validation for an unchanged candidate rather than manually repeating expensive checks without a reason.

Commit and merge hooks enforce formatting and staged whitespace only. Pre-push checks third-party notices offline when outgoing dependency or release-tooling inputs change; it does not rerun the substantive validation gates. Run substantive checks explicitly, report their candidate and results, and confirm CI passes on the published PR head before declaring it ready to merge.

Use the compiler pinned in `rust-toolchain.toml` and verify `rustc --version`
before running the gate. Standalone Cargo installations, including Homebrew's,
do not automatically select that pin; use a matching toolchain on `PATH`.

The Rust gate is:

```bash
just verify
# or: scripts/verify.sh
```

Docs-only or instruction-only changes do not require the Rust or frontend gates unless the change also affects executable examples, generated output expectations, or another validated artifact. Inspect the diff and check whitespace and any changed links or examples. See `CONTRIBUTING.md` for gate commands and hook behavior.

Complete the agreed implementation scope autonomously, including integration,
validation and documentation. Use logical commits without shrinking the requested
outcome or creating approval checkpoints for routine implementation. Request input
for unresolved consequential decisions, blockers or the early UX review described
in CONTRIBUTING.md. Scale additional independent review to risk; do not impose a
fixed multi-review ceremony on every change. Reuse valid checks for unchanged
inputs and rerun affected checks after substantive changes.

Use `just verify-changed --range <base>...HEAD --list` to discover validation
ownership. Private tooling and research datasets have their own gates; see
CONTRIBUTING.md. Unknown paths select all surfaces. CI checks every surface on
main/nightlies and selects affected surfaces for PRs.

## Commit Guidelines

Use trunk-based branches such as `feat/<topic>` or `fix/<topic>`. Commit messages must use Conventional Commits and include a summary line; a description body is optional, but when present it must be separated from the summary by a blank line.

Authorization to implement includes creating local commits at logical, validated boundaries. Do not request separate permission for each commit. Planning or design discussion alone does not authorize implementation. Explicit user instructions such as plan-only, no commits, or local-only take precedence.

Each commit should express a coherent change, include its relevant tests and documentation, and leave the branch buildable. A PR may contain several commits; commit boundaries do not dictate PR size. Avoid accumulating an entire multi-stage task uncommitted. Do not assume commit preparation must invoke a separate validation skill unless the user explicitly asks for that validation.

Do not batch unrelated changes into one commit, and do not commit half-finished work just to create progress snapshots. If the worktree already contains unrelated uncommitted changes, leave them untouched and commit only the files for the completed unit of work, or explicitly tell the user why a clean commit boundary is blocked.

## Refactor Completion Policy

Refactors must land as finished end-state changes, not as transitional scaffolding.

- Default to removing the old path outright and updating all call sites in the same task.
- Do not add compatibility layers, shim modules, fallback code paths, version bridges, or adapter wrappers unless the user explicitly asks for an incremental strategy.
- Do not treat a refactor as complete if it leaves behind intermediate shims, compatibility wrappers, transitional files, partial migrations, or mixed old-and-new implementations across the codebase.
- Validation for refactor work must explicitly check for leftover uses of the old implementation, leftover shim files, and any other signs of an intermediate state.

## Worktree Policy

Implement each task in a dedicated linked worktree on a feature branch, normally under `.worktrees/<task-slug>`. Continue using that worktree for related feedback and revisions rather than creating a new worktree for every turn. Read-only exploration may use the current checkout. Preserve unrelated changes and any running preview the user is reviewing.

Choose the base branch from the task context. For the source-modeling workstream, use `integration/source-modeling`; dependent PRs may target their preceding feature branch. Use `main` for independent work intended for the current mainline. Verify the base before editing, and do not mix unrelated workstreams.

Keep the task worktree available while its PR or local build is under review. After merge or cancellation, remove it as part of task completion once needed uncommitted work is preserved and no active preview, agent, build, or symlink depends on it. At the start of the next task, inspect existing worktrees and finish outstanding cleanup. Retaining Git history does not require retaining a checkout or its build output.

Post-merge cleanup includes synchronizing local `main`. After a task PR is confirmed merged into `main`, fetch `origin` and update the local `main` checkout with `git merge --ff-only origin/main` as part of the same cleanup that retires the task worktree. Verify that the checkout is on `main` and has no uncommitted changes before updating it, then confirm its HEAD matches the fetched `origin/main`. Do not report cleanup complete while this update remains outstanding. If local changes, divergence, or active work prevent a safe update, preserve them and report the blocked cleanup step; never reset, stash, or discard work to force synchronization. This post-merge update does not authorize merging a PR or pushing to `main`.

If a worktree must remain, record its path, purpose, owner, and removal condition in the task handoff. Preserve only the frozen binary, artifact, saved state, and small review evidence needed for a stable preview; a preview normally does not need its compiler cache. Check Git status and ignored files before removal. Never force-remove needed dirty work, user state, or another checkout's symlink target. Remove worktrees serially with `git worktree remove` and verify the resulting inventory.

These workflow rules take precedence over older skill defaults requiring separate commit permission, direct local landing, or immediate worktree cleanup.

Git commands that mutate repository state must never be run in parallel within the same repository or worktree. Serialize `git add`, `git commit`, `git rebase`, `git merge`, `git cherry-pick`, `git worktree`, and `git stash` operations.

## Disk And Build Output

Before creating a worktree or starting an expensive gate, check free space and the current task's compiler-cache size. Periodically inspect the worktree inventory rather than repeatedly scanning all archived evidence. Below 30 GiB free, reclaim this task's obsolete build output first; stop expanding caches if that cannot be done safely. A checkout target above 15 GiB is an investigation trigger, not permission to delete active files. Check growth during long builds rather than running the disk to exhaustion.

Use checkout-local Cargo output; never share or symlink targets across worktrees because tests can embed checkout-specific paths. Reuse one target per task rather than creating outputs for each agent or validation retry. Ordinary dev/test builds use limited debug information and enable incremental compilation except for the large generated atlas-foundry-model crate. Clean CI jobs disable incremental compilation globally. Keep hooks and manual checks on these defaults rather than introducing per-agent overrides. Use the documented full-debug profile when variable/type inspection is needed; retain active output and remove obsolete configurations when no longer needed. Do not routinely clean active targets to meet a size preference: rebuilding adds latency and disk writes.

Compiler caches are disposable when their task ends; saved state, source/model assets, and builds still under review require explicit preservation. Preserve live dependency targets such as `node_modules` when another checkout links to them. Use focused checks during iteration and full gates at the required milestones; reuse successful evidence for an unchanged candidate. See CONTRIBUTING.md for debugging and retention details.

For full-corpus development, use `just dev-workspace-init` to seed only a missing
checkout workspace from the shared validated baseline. Bootstrap or explicitly
refresh the retained Rust developer tool with `just dev-tool-build`; normal
baseline commands never compile. Keep the retained tool outside linked worktrees.
`just dev-status` reports
provenance and compatibility; `just dev-web` uses the frozen baseline executable
unless `--binary target/debug/atlas` explicitly selects the candidate. Share pinned
source/model caches, not writable databases or Cargo targets. Do not reset existing
saved state or run setup automatically on compatibility failure. Baseline versions
and referenced assets must survive worktree retirement; follow CONTRIBUTING.md for
publication and retention.

## Pull Requests And Human Review

For changes that affect user experience, including UI and CLI behavior or output,
provide an early functional review checkpoint. Share a runnable preview or CLI
commands with representative output as soon as the core flow works, then pause
for user feedback before broad test rewrites, full validation or independent
presentation review. Run the compilation and focused checks needed to make that
checkpoint usable, and disclose unfinished behavior and validation gaps. Preserve
existing tests; early review does not replace the required final tests, reviews
and gates after the UX direction is confirmed.

For implementation work, pushing the task branch and opening or updating a PR are part of the normal workflow unless the user requests local-only work. Confirm the intended PR base from the task context. Use a draft PR while implementation or required validation remains incomplete. Describe the resulting behavior, validation evidence, and outstanding limitations. Keep related review fixes on the same PR as additional logical commits; do not rewrite published history without agreement.

Provide a stable local build when the change is useful to inspect. Identify its branch, commit, URL or command, and any sample-data limitations; disclose uncommitted differences if present. Keep the reviewed build stable while developing further changes, using a separate checkout or built artifact when necessary. Explicitly identify when an updated build replaces it. UI work requires interaction and visual review alongside automated checks; use Astra sub-agents for presentation review, including actual browser inspection.

Do not merge PRs, push directly to shared integration branches or `main`, publish releases, or discard user state without explicit authorization. Do not use `scripts/land-worktree.sh` or `just land-worktree` as the normal landing path. Passing local checks, passing CI, and human acceptance are separate outcomes; report their status independently.

## Configuration & Data Notes

Default repo-local paths are `.cache/pf2e/<supported-commit>`, `.cache/hf-models`, and `.atlas/workspace.sqlite` with `--path-mode repo`. Global mode stores the workspace under platform application data and source/model caches under platform cache storage, both beneath `pf2e-atlas`. Normal setup updates the existing workspace and preserves saved state. Never replace or delete that database as cache cleanup. Use bounded sessions: shared reference/state reads use one snapshot, inference runs before the consuming session, and the caller owns final commit. See ADR0051 for update ownership, maintenance and recovery.
