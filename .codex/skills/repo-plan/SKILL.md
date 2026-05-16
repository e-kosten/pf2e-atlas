---
name: repo-plan
description: "Use when planning implementation work in this repository. This skill is for repo-specific planning workflow: shaping work without commits, writing a new scratch/plans file, grounding the plan in existing backlog intent or creating a new backlog item, reading the relevant architecture docs first, using sub-agent orchestration for large-task research and validation, and producing an execution-ready plan that encodes slices, docs follow-through, validation, optional worktree usage, and end-state checks."
---

# Repo Plan

Use this skill when the user wants planning work for this repository rather than immediate implementation.

This skill is specifically about how planning should happen here. It is not a generic brainstorming prompt.

## Core Workflow

1. Read the architecture docs that govern the area before planning.
   Start with:
   - `docs/architecture/overview.md`
   - `docs/architecture/boundaries.md`
   Then read the focused doc closest to the change, such as `tui.md`, `search.md`, `editorial.md`, `extending.md`, or relevant ADRs.
2. Shape the plan on `main` first.
   Planning happens in the shared checkout while the task is still being framed.
3. Write a new plan file under `scratch/plans/`.
   Do not overwrite an older plan file for a new task.
4. Tie the plan to backlog intent.
   If relevant backlog work already exists, anchor the plan to it.
   If the plan introduces a new durable follow-up or architectural intent, add or refine a backlog item once the shape is clear.
5. Keep planning uncommitted.
   Do not commit while the task is still in planning. The plan file is a working artifact.
6. Decide whether implementation should stay in the current checkout or use an explicit `$worktree` checkout. Use a worktree when the plan needs parallel agents, risky isolation, or a preserved main checkout; otherwise plan for the current checkout.

## What the Plan Must Contain

A good plan in this repo is execution-ready. It should usually include:

- the requested end state in concrete terms
- architectural boundaries and owners that matter
- explicit slices or workstreams
- which parts are local work versus delegated work
- validation ownership for each slice
- required docs and ADR follow-through
- refactor end-state requirements when replacing shared infrastructure
- checkout, optional worktree, and landing expectations
- blockers, assumptions, and open questions

Do not produce a vague task list that leaves architecture, ownership, or validation implicit.

When the task is large, architectural, or contract-shaping, make the end state concrete enough that later implementation does not have to rediscover the intended model. Good ways to do that include:

- a representative target shape for a new shared contract or model
- an explicit list of resolved planning decisions
- change-envelope rules that say what is allowed to move and what should stay stable during the refactor
- clear user check-in triggers if the work would need to expand beyond that envelope

## Orchestration Expectations

For large or architecture-impacting work, plan as an orchestrator:

- break the task into explicit slices with dependencies
- identify which slices are good candidates for sub-agent research, implementation, or validation
- keep the main agent focused on coordination and end-state checks
- make each slice produce a concrete artifact and a validation step
- when helpful, group slices into orchestration blocks with explicit check-in points so the user can review progress between major phases
- if one slice is still too large to be implementation-ready, split it into sub-slices with clear ownership and validation boundaries

Use sub-agents for research and review when the task is large enough to benefit from parallel or isolated investigation. Planning should not assume one agent will carry all context alone if the work naturally decomposes.

## Backlog Coupling

Plans should not float separately from project intent.

- If the task is already represented in `docs/backlog/`, cite that item and align the plan with it.
- If planning reveals a missing durable follow-up, add a backlog item for it.
- If planning changes the shape of an existing backlog item, update the backlog so it matches the intended work.

Do not leave durable follow-up work implied only in the plan file.

## Checkout and Worktree Boundary

Use this boundary deliberately:

- allowed in the current checkout during planning:
  - reading code and docs
  - writing a new `scratch/plans/...` file
  - shaping or revising backlog intent while the work is still being framed
- allowed in the current checkout during implementation:
  - small or serial tracked edits where checkout isolation is unnecessary
  - follow-up fixes where no other agent is editing the same files
- prefer an explicit `$worktree` checkout when:
  - multiple agents need to edit tracked files concurrently
  - the task is risky, long-running, or likely to pause mid-state
  - the user wants the main checkout preserved
  - the plan needs isolated validation or landing from a branch

If the plan chooses a worktree, state the reason and invoke `$worktree` before tracked implementation begins.

## Required Plan Sections

Prefer plans with sections close to these:

1. Summary
2. Architecture context
3. Execution model or orchestration breakdown
4. Implementation slices
5. Docs and ADR updates
6. Validation plan
7. Landing workflow
8. Assumptions, blockers, and open questions

If the task is small, some sections can be brief. For large work, all of them should be present.

For especially large or model-heavy work, it is often worth adding a few more explicit sections near the top instead of burying critical decisions inside slices:

1. Target model or representative end-state shape
2. Resolved planning decisions
3. Change envelope or stability rules

Use these when they materially reduce ambiguity. Do not add them as ceremony for small plans.

## Docs Expectations

If the plan touches durable structure, the plan should call out the docs that must stay in sync.

At minimum, consider:

- relevant files under `docs/architecture/`
- related backlog item updates
- ADR updates when the plan introduces a new durable rule or architectural choice future editors must preserve

Do not treat docs updates as optional cleanup for architecture work.

## Refactor Planning Rules

When the plan is for a refactor:

- plan for direct replacement, not a compatibility layer, unless the user explicitly wants an incremental migration
- include a check for leftover old paths, adapters, shims, or mixed implementations
- define the end state as complete replacement, not partial adoption

If the refactor cannot land cleanly without an intermediate compatibility layer, flag that as a blocker in the plan instead of quietly normalizing it.

## Validation Requirements

Every implementation plan should define how completion will be proven.

Include:

- targeted validation for each slice where practical
- final validation commands
- plan-file validation before reporting completion
- checks that docs and code agree
- checks that no intermediate migration state remains

For larger plans, do not stop at command lists. Also include invariant-style validation, such as:

- semantic or architectural properties that must be true after each slice
- “no parallel abstraction remains” checks for refactors that are removing duplicated owners
- explicit checks that route/state/presentation projections remain derived rather than becoming new durable models
- surface-specific checks when a shared abstraction is being introduced and later consumed by multiple surfaces

If the plan came from a `scratch/plans/` file, the final implementation pass must validate against that file before claiming completion.

## Good Planning Defaults

- prefer exact file and module references when known
- prefer concrete acceptance criteria over broad goals
- prefer explicit follow-up tracking over “later” prose
- pause and ask the user if a blocker or architecture question prevents a clean end-state plan
- keep plan prose concise, but make ownership and validation explicit
- for very large plans, prefer explicit “what this plan is not changing” guidance so implementation does not sprawl
- when the intended model is non-obvious, include a representative shape or examples rather than leaving the meaning implicit in prose alone
- when a successful implementation will depend on a few non-negotiable architectural rules, state them directly in the plan instead of assuming they are obvious from the slice text

## Output Shape

Unless the user asks for something else, the planning output should usually produce:

- a short framing summary in chat
- a new `scratch/plans/...` file with the full execution plan
- backlog updates when the task needs durable intent captured outside the plan

The plan should be strong enough that a later implementation turn can execute from it without re-deriving the architecture.
