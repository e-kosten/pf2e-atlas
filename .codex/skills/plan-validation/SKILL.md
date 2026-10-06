---
name: plan-validation
description: Skeptical post-implementation validation for completed plan files or outcome-scoped implementation briefs in this repository. Use when a user wants an independent audit of the actual diff and production paths, especially to find missed outcomes, weak validation, architecture contradictions, incomplete refactors, or incorrectly reported completion.
---

# Plan Validation

Use this skill after implementation, not before. Treat the latest outcome-scoped brief plus its semantic plan/architecture requirements as the validation contract, and test whether the repository actually reached that end state. Procedural allowlists or superseded publication steps do not override the current brief, but semantic invariants and acceptance criteria remain binding unless explicitly changed by the appropriate decision owner.

The goal is to find every credible gap before the user is told the plan is complete. Do not rely on the previous orchestrator's summary, commit message, or green tests as proof of completion.

## Required Input

- Require a concrete plan file path or an equally concrete outcome-scoped implementation brief.
- If the plan references review artifacts, addenda, backlog items, architecture docs, ADRs, stop points, or validation commands, include those in the validation contract.
- If the target implementation is in a branch, worktree, commit, or diff range, identify that state before delegating.

If the user asks for validation without a plan path or concrete scope, ask for the missing path or scope before doing substantial work.

## Core Workflow

1. Read the full plan contract.
   Extract every promised outcome, validation requirement, docs/backlog requirement, architecture invariant, non-goal, and explicit decision that changed acceptance.
2. Build a validation matrix.
   Convert the semantic contract into small, checkable items. Each item needs an expected state, likely production path or owner to inspect, and evidence that would prove it is satisfied. Treat file lists as leads, not coverage boundaries.
3. Split the matrix into tightly verifiable focus areas.
   Prefer coherent risk/behavior boundaries that a validator can fully inspect in one pass. Do not manufacture a separate area for every file or procedural step.
4. Perform skeptical independent validation.
   Inspect the actual diff and production call paths. Use independent validators when they add real separation or expertise; do not delegate every checklist row by default.
5. Run cross-cutting validation passes.
   Cover validation evidence, architecture/docs follow-through, refactor completion, baseline-versus-regression status, and git/worktree obligations when relevant.
6. Aggregate the findings into one evidence-backed verdict.
   Do not summarize away gaps. Batch related findings by failed boundary so one remediation pass can address them coherently. Create a durable validation artifact only for a large or durable audit, multiple validators whose evidence needs aggregation, or an explicit user request.
7. Give the user a verdict.
   Say whether the plan is fully satisfied. If not, name the blocking gaps. Link a validation artifact only when one was warranted.

## Focus Area Design

A good focus area is small enough that a validator can say "complete" or "not complete" with concrete file references.

Use these defaults when the distinctions reflect separate risks:

- One focus area per plan block when blocks are already narrow.
- Split a block when it spans independently risky ownership boundaries, such as search, UI, data indexing, or server tools.
- Group docs, ADRs, backlog/history movement, validation commands, and landing/worktree obligations with the product or architecture boundary they support unless they present an independent risk.
- Create a separate refactor-cleanup focus area for any plan that replaces an old path, removes a facade, introduces an abstraction, or forbids shims.
- Create a separate acceptance-behavior focus area when the plan promises user-visible behavior that tests might not prove.

Do not create a vague "general review" area unless it is only an extra final sweep after all concrete items have owners.

## Delegation

Use read-only independent validators when the task's risk, breadth, or need for separation justifies them, unless the user explicitly asks for remediation. The top-level agent owns the risk-area split. Validators inspect files, production paths, run read-only searches, and run permitted validation commands when useful, but do not patch code.

Seed every delegated validator with `$delegated-agent-contract`. Validation agents must receive a concise outcome-scoped brief, not a vague request to "look deeply". If the focus area cannot be described with enough outcome, implementation state, architecture context, invariants, non-goals, and validation evidence expectations, tighten the focus area before spawning the validator.

For each delegated focus area, provide:

- the plan file path
- the outcome and semantic plan items assigned
- any referenced addenda or review artifacts
- the implementation state to inspect, such as current worktree, branch, commit, or diff range
- likely files, modules, production paths, docs, or commands to check as leads rather than an exhaustive boundary
- a reminder to validate against the plan, not against the previous completion claim
- relevant architecture docs, ADRs, lint rules, ownership boundaries, and explicit no-go shortcuts for the focus area
- a reminder to return `brief incomplete` instead of guessing when the caller did not provide enough context to validate safely
- the required report format below

When focus areas are independent and parallelism materially helps, validators may run in parallel. A second validator is warranted when a high-risk boundary needs genuinely independent evidence or different expertise, not automatically for every architecture change or final verdict.

If an independent validator is unavailable, perform the same passes locally and disclose that limitation in the verdict or artifact. Do not pretend local passes had independent-agent coverage.

## Validator Report Format

Ask every validator to return:

- `verdict`: `pass`, `fail`, or `uncertain`
- `items checked`: the assigned plan items
- `evidence`: concrete files, line references, commands, tests, or searches used
- `gaps`: missing, partial, contradicted, or unproven items
- `risk`: anything that could make the plan unsafe to call complete
- `recommended follow-up`: the smallest remediation or extra validation needed

Also require the delegated-agent contract fields:

- `architecture concerns`: shortcut-looking implementations, owner drift, shims, boundary bypasses, duplicated shared logic, or `none found`
- `contract issues`: missing assignment inputs that limited confidence, or `none`

Require validators to distinguish:

- not implemented
- partially implemented
- implemented differently than the plan
- implemented but not validated
- validation command missing or failing
- docs or ADR follow-through missing
- old path, shim, compatibility layer, or mixed ownership left behind
- plan ambiguity that prevents a confident verdict
- pre-existing baseline failure
- candidate-introduced or worsened regression
- reused evidence whose relevant inputs changed or whose limitations were not disclosed

## Cross-Cutting Passes

Run these in addition to plan-block validation when applicable:

- **Contract coverage**: every checklist item, explicit acceptance criterion, and stop point has a verdict.
- **Validation evidence**: claimed build, tests, refresh commands, generated artifacts, or manual checks actually support the completed scope.
- **Production-path behavior**: tests and inspection cover the real caller/writer/reader/transport/runtime route rather than only declarations or synthetic DTOs.
- **Baseline treatment**: pre-existing failures are identified separately, remain visible, and are not silently accepted as candidate success.
- **Architecture consistency**: relevant `docs/architecture/` and ADR guidance match the implementation, and required doc updates landed.
- **Refactor completion**: old implementations, compatibility wrappers, temporary shims, unused exports, and mixed old/new call paths are gone.
- **Search/index/data implications**: required refresh steps, schema changes, fixtures, generated indexes, and migration notes are accounted for.
- **Git/worktree completion**: expected commit, merge, branch, validation on main, and temporary worktree cleanup obligations are satisfied when the plan or repo policy requires them.

## Validation Output

Ordinary validation may return the concise evidence-backed verdict above directly in chat or an AO callback. Do not create a scratch artifact, checksum package, or multi-section matrix solely because validation occurred.

Write a new artifact under the following path only when the audit is large, intended as durable evidence, aggregates multiple validators, or the user requests it:

- `scratch/plan-validation/YYYY-MM-DD-<topic>-validation.md`

Do not overwrite an existing validation artifact for a new pass. Treat the artifact as scratch output unless the user explicitly asks to commit it.

When an artifact is warranted, include only the sections needed to preserve the evidence. A large multi-validator audit will usually need:

1. Plan and implementation scope
2. Overall verdict
3. Validation matrix
4. Independent focus areas and any delegated validators
5. Findings by focus area
6. Cross-cutting findings
7. Missing, partial, or unproven plan items
8. Recommended remediation order
9. Commands or checks run

If a checklist matrix materially improves coverage, use:

- `[x]` fully satisfied with evidence
- `[ ]` missing, partial, contradictory, or unproven
- `[-]` not applicable, with a short reason

## Aggregation Rules

- A plan is not complete if any required item is missing, partial, contradictory, or unproven.
- A green test suite does not close a plan item unless the item was actually covered by the test or another explicit check.
- Prior evidence closes an item only when all relevant source, toolchain, candidate, generator, and policy inputs are unchanged and the reuse limitation is reported.
- "Implemented but not validated" is a gap, not a pass.
- "Mostly done" is a fail for the relevant item.
- A validator's uncertainty is a gap unless the plan item is non-required or the uncertainty can be resolved locally with evidence.
- If validators disagree, inspect the specific evidence and record the adjudication. Do not average the results.
- Batch findings that share a cause or production boundary; do not serialize avoidable one-finding remediation cycles.
- Do not fix issues during validation unless the user asks for remediation. The output of this skill is the gap report and verdict.

## User-Facing Output

Keep chat or the callback concise:

- state the overall verdict
- give the artifact path when one was created
- list the highest-impact gaps, if any
- say what independent validation was used and disclose any independence limitation
- mention commands that were run or any validation that could not be run

If the verdict is fail, do not describe the implementation as complete. Give the smallest remediation order in the concise verdict or, when one exists, the artifact.
