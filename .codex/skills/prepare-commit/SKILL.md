---
name: prepare-commit
description: Use when the user says to prepare a commit or identifies a committable milestone. Runs a fresh two-validator review loop over the candidate commit scope, fixes reported issues, repeats with fresh agents until clean, then creates the commit.
---

# Prepare Commit

Use this skill only when the user explicitly asks to prepare a commit, create a commit, or says the current work has reached a committable milestone.

This skill replaces automatic end-of-task commits. The goal is to keep implementation iteration separate from the final commit-readiness gate.

## Scope

Before validation, define the candidate commit scope:

- identify the active task worktree and branch
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

Run validation in cycles. Every cycle must use fresh validation agents; do not reuse validators from earlier dirty cycles.

In each cycle:

1. Spawn two read-only validators in parallel with fresh context. Prefer `fork_context: false` and pass only the planning snapshot, candidate commit scope, relevant files, and assignment instructions.
2. Wait for both reports.
3. If either validator reports `fail` or actionable `uncertain`, fix the issues in the task worktree.
4. Run any targeted checks needed for the fixes.
5. Start a new cycle with fresh validators.
6. Continue until both fresh validators return `pass`, or until a blocker requires user input.

Close stale validator agents after each failed cycle when the runtime supports it. Their context contains obsolete defect history and must not be treated as the final clean review. Do not include previous validator reports in the next cycle's prompt except for a short factual note that fixes were made and the current diff is the source of truth.

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

## Remediation

The main agent owns fixes. Validators should stay read-only unless the user explicitly asks otherwise.

When fixing validator findings:

- keep edits inside the active task worktree
- do not broaden the commit scope without telling the user
- preserve unrelated changes
- update tests and docs when the fix changes behavior or guidance
- rerun targeted validation locally before starting the next fresh-validator cycle

If the validators disagree, inspect their evidence and either fix the underlying issue or explain why one finding is not applicable in the next cycle's context.

## Final Validation and Commit

After both fresh validators pass:

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

Do not commit if either fresh validator did not pass, if required validation fails, or if unrelated changes prevent a clean commit boundary.
