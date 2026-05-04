---
name: plan-validation
description: Deep post-implementation validation for completed plan files in this repository. Use when a user wants an independent audit after plan-orchestration or implementation work, especially to find plan items that were missed, partially implemented, poorly validated, contradicted by architecture docs, left as transitional refactor state, or incorrectly reported complete. Requires a concrete plan file path or an explicit completed implementation scope to validate.
---

# Plan Validation

Use this skill after implementation, not before. Treat the plan and any referenced addenda as the execution contract, and test whether the repository actually reached that end state.

The goal is to find every credible gap before the user is told the plan is complete. Do not rely on the previous orchestrator's summary, commit message, or green tests as proof of completion.

## Required Input

- Require a concrete plan file path unless the user gives an equally concrete implementation contract.
- If the plan references review artifacts, addenda, backlog items, architecture docs, ADRs, stop points, or validation commands, include those in the validation contract.
- If the target implementation is in a branch, worktree, commit, or diff range, identify that state before delegating.

If the user asks for validation without a plan path or concrete scope, ask for the missing path or scope before doing substantial work.

## Core Workflow

1. Read the full plan contract.
   Extract every promised deliverable, validation requirement, stop point, docs/backlog requirement, architecture constraint, and explicit "do not" rule.
2. Build a validation matrix.
   Convert the contract into small, checkable items. Each item needs an expected state, likely files or modules to inspect, and evidence that would prove it is satisfied.
3. Split the matrix into tightly verifiable focus areas.
   Prefer narrow areas that a validator can fully inspect in one pass. Split by plan block, subsystem, or obligation type rather than by broad theme.
4. Delegate deep validation for every focus area.
   Use subagents by default. Each validator must read the plan excerpt plus relevant files and report whether the assigned items are fully satisfied.
5. Run cross-cutting validation passes.
   Always cover validation evidence, architecture/docs follow-through, refactor completion state, and git/worktree completion obligations when they are relevant.
6. Aggregate the findings into a validation artifact.
   Do not summarize away gaps. Record every missing, partial, contradictory, or unproven item.
7. Give the user a verdict.
   Say whether the plan is fully satisfied. If not, name the blocking gaps and point to the validation artifact.

## Focus Area Design

A good focus area is small enough that a validator can say "complete" or "not complete" with concrete file references.

Use these defaults:

- One focus area per plan block when blocks are already narrow.
- Split a block when it spans multiple ownership boundaries, such as search, TUI, data indexing, server tools, docs, or tests.
- Create separate focus areas for docs, ADRs, backlog/history movement, validation commands, and landing/commit/worktree obligations when the plan mentions them.
- Create a separate refactor-cleanup focus area for any plan that replaces an old path, removes a facade, introduces an abstraction, or forbids shims.
- Create a separate acceptance-behavior focus area when the plan promises user-visible behavior that tests might not prove.

Do not create a vague "general review" area unless it is only an extra final sweep after all concrete items have owners.

## Delegation

Use read-only subagents for validation unless the user explicitly asks for remediation. The top-level agent using this skill should own the focus-area split and spawn the validators directly when subagent tools are available. Validators should inspect files, run read-only searches, and run permitted validation commands when useful, but they should not patch code.

Seed every delegated validator with `$delegated-agent-contract`. Validation agents must receive a complete assignment contract, not a vague request to "look deeply". If the focus area cannot be described with enough plan scope, implementation state, architecture context, ownership, forbidden shortcuts, and validation evidence expectations, tighten the focus area before spawning the validator.

For each focus area, provide:

- the plan file path
- the exact plan items assigned
- any referenced addenda or review artifacts
- the implementation state to inspect, such as current worktree, branch, commit, or diff range
- likely files, modules, docs, or commands to check
- a reminder to validate against the plan, not against the previous completion claim
- relevant architecture docs, ADRs, lint rules, ownership boundaries, and explicit no-go shortcuts for the focus area
- a reminder to return `contract incomplete` instead of guessing when the caller did not provide enough context to validate safely
- the required report format below

When the focus areas are independent, spawn validators in parallel. Use two validators for high-risk or cross-cutting areas such as architecture boundary changes, refactors, and final whole-plan verdicts.

If subagents are unavailable in the current runtime, perform the same passes locally and mark the validation artifact with that limitation. Do not pretend local passes had independent-agent coverage.

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

## Cross-Cutting Passes

Run these in addition to plan-block validation when applicable:

- **Contract coverage**: every checklist item, explicit acceptance criterion, and stop point has a verdict.
- **Validation evidence**: claimed build, tests, refresh commands, generated artifacts, or manual checks actually support the completed scope.
- **Architecture consistency**: relevant `docs/architecture/` and ADR guidance match the implementation, and required doc updates landed.
- **Refactor completion**: old implementations, compatibility wrappers, temporary shims, unused exports, and mixed old/new call paths are gone.
- **Search/index/data implications**: required refresh steps, schema changes, fixtures, generated indexes, and migration notes are accounted for.
- **Git/worktree completion**: expected commit, merge, branch, validation on main, and temporary worktree cleanup obligations are satisfied when the plan or repo policy requires them.

## Validation Artifact

Write a new artifact under:

- `scratch/plan-validation/YYYY-MM-DD-<topic>-validation.md`

Do not overwrite an existing validation artifact for a new pass. Treat the artifact as scratch output unless the user explicitly asks to commit it.

Include:

1. Plan and implementation scope
2. Overall verdict
3. Validation matrix
4. Delegated focus areas and assigned validators
5. Findings by focus area
6. Cross-cutting findings
7. Missing, partial, or unproven plan items
8. Recommended remediation order
9. Commands or checks run

Use checkboxes in the validation matrix:

- `[x]` fully satisfied with evidence
- `[ ]` missing, partial, contradictory, or unproven
- `[-]` not applicable, with a short reason

## Aggregation Rules

- A plan is not complete if any required item is missing, partial, contradictory, or unproven.
- A green test suite does not close a plan item unless the item was actually covered by the test or another explicit check.
- "Implemented but not validated" is a gap, not a pass.
- "Mostly done" is a fail for the relevant item.
- A validator's uncertainty is a gap unless the plan item is non-required or the uncertainty can be resolved locally with evidence.
- If validators disagree, inspect the specific evidence and record the adjudication. Do not average the results.
- Do not fix issues during validation unless the user asks for remediation. The output of this skill is the gap report and verdict.

## User-Facing Output

After the artifact is written, keep chat concise:

- state the overall verdict
- give the artifact path
- list the highest-impact gaps, if any
- say whether subagent validation covered every focus area
- mention commands that were run or any validation that could not be run

If the verdict is fail, do not describe the implementation as complete. Offer a remediation order in the artifact rather than turning the chat response into a second long report.
