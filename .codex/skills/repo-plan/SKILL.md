---
name: repo-plan
description: Create or refine a concise repository plan for substantial implementation or research, connecting scope and acceptance to architecture and backlog. Reuse the current task plan; routine changes usually need no plan document.
---

# Repo Plan

Use a written plan when design decisions, dependent work or a durable handoff
justify one. Match its structure and detail to what the next executor needs.

## Ground the task

Start with the user's scope, the local `scratch/plans/README.md` when present,
and the related backlog item. Reuse an existing task plan while its deliverable
evolves; create a separate plan when the workstream or outcome is distinct.

Verify the relevant checkout and baseline. Follow `AGENTS.md` for Git workflow,
authorization and validation. Read the architecture documents governing the task,
starting with `docs/architecture/overview.md` when ownership or boundaries matter.

## Make the plan actionable

Keep one current plan per substantial task, normally under
`scratch/plans/YYYY-MM-DD-topic.md`. Describe the outcome, scope, relevant owners,
ordered work and acceptance checks. Include unresolved decisions, dependencies,
resource limits and delivery details where they affect execution.

Choose sections that make the work clear. Link existing repository guidance and
research evidence; focus the plan itself on task-specific decisions and commitments.
For delegated work, define ownership, shared contracts and coordination points.
Choose review and validation depth according to the change's risk.

Connect durable open work to the backlog. Include architecture/doc updates among
the deliverables when the task changes ownership or intended behavior. Refactor
acceptance should describe the intended end state and how it will be verified.

## Maintain status and navigation

Use a small header:

```text
Status: proposed | in_progress | deferred | complete | superseded | needs_review
Updated: YYYY-MM-DD
Backlog: path, when applicable
```

Keep current and deferred plans in `scratch/plans/README.md`, with status and the
next action. Update the same plan as decisions evolve. Preserve a prior version
when an external reference or recorded evidence depends on that version.

On completion or supersession, archive local plans under
`scratch/plans/archive/YYYY-MM/` and tracked plans under
`docs/backlog/history/plans/`. Record their status, update navigation and preserve
evidence provenance. Use `needs_review` when historical completion is unverified.
Carry remaining durable work into the backlog.

## Finish

Check that the plan captures the agreed outcome, executable next steps, acceptance
criteria and open decisions. Report the result and material choices briefly.
Distinguish approval of a plan from authorization to execute or publish the work.
