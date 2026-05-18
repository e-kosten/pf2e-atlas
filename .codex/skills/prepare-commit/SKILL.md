---
name: prepare-commit
description: Use when the user says to prepare a commit or identifies a committable milestone. Runs plan, implementation-quality, and SRP/giant-file validators over the candidate commit scope, uses the same validators to re-check reported fixes, then requires a final fresh-validator pass before creating the commit.
---

# Prepare Commit

Use this skill only when the user explicitly asks to prepare a commit, create a commit, or says the current work has reached a committable milestone.

This skill replaces automatic end-of-task commits. The goal is to keep implementation iteration separate from the final commit-readiness gate.

## Scope

Before validation, define the candidate commit scope:

- identify the active checkout, branch, and whether the work is in a task worktree
- inspect `git status --short` and the diff against the intended base, usually `main`
- identify staged, unstaged, and untracked files that belong to this milestone
- exclude unrelated user or agent changes from the commit scope
- record any plan file, backlog item, architecture doc, ADR, or explicit user acceptance criteria that shaped the work

If the commit scope is unclear or mixed with unrelated changes, stop and ask the user how to split it.

## Planning Context Snapshot

Create a planning-context snapshot for validators before spawning them. It must represent the agreement before implementation started, not the main agent's later self-assessment.

Include:

- the user's original request and any follow-up decisions before work began
- plan files or checklist items, with paths when present
- architecture or boundary constraints that were part of the agreement
- explicit non-goals, forbidden shortcuts, or required validation
- the candidate diff range, branch, and files under review

Prefer full pre-implementation excerpts from the conversation, plan file, or checklist over summaries when they are available. When the exact pre-implementation conversation boundary is not recoverable, say so in the snapshot and use the best available plan artifacts and user instructions. Do not use implementation summaries as a substitute for the plan.

## Validator Loop

Run validation in two phases: an issue-fix loop that reuses the same validators for efficient rechecks, then a final clean pass with fresh validators.

### Issue-Fix Loop

Start by spawning three read-only validators in parallel with fresh context: plan completion, implementation quality, and SRP/giant-file review. Prefer `fork_context: false` and pass only the planning snapshot, candidate commit scope, relevant files, and assignment instructions.

For each validator report:

1. Wait for all three reports.
2. If all validators return `pass`, leave the issue-fix loop and start the fresh final pass.
3. If any validator reports `fail` or actionable `uncertain`, fix the issues in the active task checkout.
4. Run any targeted checks needed for the fixes.
5. Send the relevant fix summary, changed files, and updated diff context back to the same validator agents.
6. Ask each prior validator to re-check its own findings and any directly related regressions.
7. Continue this same-validator recheck loop until all prior validators return `pass`, or until a blocker requires user input.

Reusing validators is intentional during remediation: they already understand their findings and can verify whether the fix addresses the exact problem without rebuilding context from scratch.

### Fresh Final Pass

After the same-validator loop passes, spawn a new set of three read-only validators with fresh context. These final validators must not inherit context from the dirty remediation loop.

The final fresh validators should receive the planning-context snapshot, candidate commit scope, current diff, relevant files, and assignment instructions. Do not include prior validator reports except for a short factual note that earlier issues were fixed and the current diff is the source of truth.

If any final fresh validator reports `fail` or actionable `uncertain`, fix the issues, use those final validators for the next same-validator issue-fix loop, then run another fresh final pass. The commit gate is satisfied only when a fresh set of all three validators returns `pass`.

Close stale validator agents after each loop when the runtime supports it. Dirty-loop validator context is useful for rechecking its own findings, but it must not be treated as the final clean review.

## Validator 1: Plan Completion

Give this validator the planning-context snapshot. Its job is to validate the implementation against the agreed plan, independent of the main agent's completion claims.

Ask it to check:

- every planned deliverable is fully landed
- no agreed plan item is missing, partial, deferred, or hidden as follow-up
- docs, ADRs, backlog movement, and validation obligations from the plan are satisfied
- refactors reached the requested end state without shims, compatibility layers, or mixed old/new paths
- the candidate commit scope matches the milestone the user asked to prepare

Required report format:

- `verdict`: `pass`, `fail`, or `uncertain`
- `plan items checked`
- `evidence`
- `gaps`
- `shortcuts or incomplete work`
- `recommended fixes`

## Validator 2: Implementation Quality

Give this validator the candidate diff range, changed files, and relevant architecture docs. It should review the work on engineering quality even if the plan is satisfied.

Ask it to check:

- duplication or parallel behavior that should have reused existing abstractions
- architecture documentation, ownership boundaries, and lint-enforced boundaries
- test quality, including whether tests prove intent instead of restating implementation details
- error handling, edge cases, type safety, naming, module placement, and maintainability
- docs and user-facing guidance accuracy for the final behavior
- whether validation commands are appropriate for the blast radius

Required report format:

- `verdict`: `pass`, `fail`, or `uncertain`
- `files reviewed`
- `evidence`
- `quality findings`
- `test concerns`
- `architecture concerns`
- `recommended fixes`

## Validator 3: SRP And Giant Files

Give this validator the candidate diff, changed file list with line counts, relevant architecture docs, and any repository-specific module-size or ownership rules. Its job is to find files, modules, tests, or abstractions that are too large, own too many responsibilities, or hide unrelated policies in one place.

Ask it to check:

- new or materially expanded files over roughly 700 lines, plus any smaller file that still mixes multiple responsibilities
- modules that combine orchestration, policy, storage/schema details, runtime execution, presentation, validation, fixtures, or unrelated helpers
- generic `utils`, `helpers`, registry, facade, or catch-all modules that should be split by domain concern
- tests that became giant fixture bins or combine unrelated behavior checks in a way that obscures intent
- whether long declarative registries are truly one concern, or whether they should be partitioned by family, table, command, or policy owner
- whether splitting would improve ownership without creating shims, compatibility layers, or artificial indirection

Required report format:

- `verdict`: `pass`, `fail`, or `uncertain`
- `files reviewed with line counts`
- `single-responsibility findings`
- `giant-file findings`
- `split recommendations`
- `non-blocking observations`

## Remediation

The main agent owns fixes. Validators should stay read-only unless the user explicitly asks otherwise.

When fixing validator findings:

- keep edits inside the active task checkout
- do not broaden the commit scope without telling the user
- preserve unrelated changes
- update tests and docs when the fix changes behavior or guidance
- rerun targeted validation locally before asking validators to re-check

If the validators disagree, inspect their evidence and either fix the underlying issue or explain why one finding is not applicable in the next cycle's context.

## Final Validation and Commit

After all three fresh validators pass:

1. Run the repository validation appropriate to the final scope.
   For implementation changes, default to:
   - `cd scripts && npm run lint`
   - `npm run build`
   - `cd scripts && npm test`
   Docs-only or instruction-only changes do not require build/test unless the edited files require it.
2. If final validation requires code, test, or doc changes, make the fix and return to the fresh-validator loop before committing.
3. Inspect `git status --short`.
4. Stage only files in the approved commit scope.
5. Commit with a Conventional Commit message.
6. Report the commit SHA, commit message, validators' pass verdicts, commands run, skipped validation with reason, and any required refresh or migration follow-up.

Do not commit if any fresh validator did not pass, if required validation fails, or if unrelated changes prevent a clean commit boundary.
