---
name: repo-plan
description: Create or refine a concise repository plan for substantial implementation or research, connecting scope and acceptance to architecture and backlog. Reuse the current task plan; routine fixes and conversational clarifications need no plan document.
---

# Repo Plan

Use a written plan when work has meaningful design decisions, several dependent
steps, or enough duration that a durable handoff helps. Handle routine changes
directly; a question about the next step does not automatically need a new file.

## Find the current task

- Start with the local `scratch/plans/README.md` when present, the related backlog
  item and the user's current instructions. Search for an existing task plan before
  creating one. Follow-ups, revised proposals and execution notes belong in that
  plan unless the deliverable or workstream materially changes.
- Work in the current checkout during planning. Authenticate the relevant checkout
  and baseline; preserve unrelated dirt and running previews. Never switch to
  `main` merely to write a plan. Use a worktree for implementation only when the
  user asks or isolation has a concrete benefit.
- Read the relevant architecture docs before deciding ownership. Start with
  `docs/architecture/overview.md`, then the focused docs/ADRs that exist and govern
  the task. Do not require imaginary or retired document paths.

## Write only what execution needs

Keep one active plan per substantial task, normally under
`scratch/plans/YYYY-MM-DD-topic.md`. Prefer a short document; a few paragraphs and
an ordered list often suffice. Expand only for decisions, contracts or dependencies
that the executor would otherwise have to rediscover. Do not impose eight sections,
restate general repo instructions or create separate assignment/ledger plans by
default. Detailed evidence belongs in the task's research/review directory.

Include the outcome, scope, relevant owners, ordered work and acceptance checks.
Add unresolved decisions, resource limits, delegation or landing details only when
they matter. Tie durable open work to an existing backlog item, creating/updating
one only within the user's scope. Architecture docs describe current ownership;
plans describe intended work. Plan relevant docs/ADR updates with implementation.

For complex work, make shared contracts and stop points concrete. Delegate when
independent research/review or parallel work adds value, not as a mandatory plan
approval step. Refactors should define a finished replacement and checks for old
paths; avoid planning compatibility shims unless requested.

## Keep plans discoverable

Use a small header on current plans:

```text
Status: proposed | in_progress | deferred | complete | superseded | needs_review
Updated: YYYY-MM-DD
Backlog: path, when applicable
```

Keep `scratch/plans/README.md` as a small index of current/proposed/deferred work,
with status and the next action. Preserve user authorization separately from status:
approval of planning or a cleanup does not approve implementation or publication.

On completion or supersession, move the plan to `scratch/plans/archive/YYYY-MM/`,
update the index and mutable links, and carry durable remaining work to the backlog.
Keep previously tracked historical plans in `docs/backlog/history/plans/` so their
relocation remains Git-visible rather than disappearing into ignored scratch.
Archive uncertain historical plans as `needs_review`, never infer completion from
age. Retain provenance-bound originals when relocation/link edits affect recorded
hashes; do not rewrite frozen evidence or leave broken current navigation. Ordinary
plan revisions do not need a new dated file or an archived snapshot.

## Finish the planning task

Check that the plan captures agreed scope and can guide the next work. Scale review
to risk; simple plans do not need independent reviewers or implementation test runs.
Present the outcome and material open choices briefly. Keep planning uncommitted
unless asked; do not infer authority to execute, commit, publish or merge from it.
