---
name: plan-analysis
description: "Use when the user wants a detailed analysis or critique of an existing plan file in this repository before implementation. This skill is for reading the plan, comparing it against current architecture docs, backlog intent, live abstractions, and code owners, writing a separate review artifact under scratch/plan-review/, and then working through the findings one by one instead of short-circuiting after the first few issues."
---

# Plan Analysis

Use this skill when the user wants to review or critique an existing plan rather than create a new execution plan.

This skill is specifically about full-pass plan analysis in this repository. It is not a generic brainstorming prompt and it is not an implementation workflow.

## Core Workflow

1. Read the target plan file first.
   Identify the requested end state, the plan’s major slices, and any explicit assumptions or architecture claims.
2. Read the governing architecture docs before judging the plan.
   Start with:
   - `docs/architecture/overview.md`
   - `docs/architecture/boundaries.md`
   Then read the focused doc closest to the plan area, such as `search.md`, `tui.md`, `editorial.md`, `extending.md`, or relevant ADRs.
3. Inspect the live code owners and abstractions the plan would need to build on.
   Prefer concrete file/module checks over abstract guesses.
4. Complete the full review pass before surfacing detailed findings in chat.
   Do not stop after identifying the first few issues. The expected output is a finished review artifact, not an early verbal partial.
5. Write a new review file under `scratch/plan-review/`.
   Do not overwrite older review files for a new review task.
6. Keep the original plan separate from the review artifact.
   Analyze first in `scratch/plan-review/`; only tighten the original plan if the user asks.
7. Expect an interactive follow-up.
   After writing the review file, discuss findings one by one with the user and update the review checklist as items are resolved or intentionally dismissed.

## Review Artifact Requirement

The default output of this skill is a new file under:

- `scratch/plan-review/YYYY-MM-DD-<topic>-review.md`

Use `scratch/plan-review/`, not `scratch/plans/`, for review artifacts.
These review artifacts are transient scratch files. Create the directory on demand if it does not exist, and do not commit the review artifact unless the user explicitly asks.

The review file should include:

1. Summary
2. Review scope
3. Docs and code owners consulted
4. Findings checklist
5. Detailed findings
6. Open questions or non-issues
7. Recommended plan tightenings

Use a findings checklist with Markdown checkboxes so the review can be worked through interactively:

- `[ ]` finding still open for discussion or action
- `[x]` finding discussed and resolved, or explicitly dismissed as not requiring a plan change

Prefer one checklist line per substantive finding. Keep them short enough to scan quickly.

If useful, add a short “resolution” note under the detailed finding after it is discussed.

## Chat Behavior

After the review file exists:

- give a short framing summary in chat
- say where the review artifact lives
- do not dump every finding unless the user asks
- assume the user will want to go one finding at a time

When the user asks for the “first” or “next” finding:

- answer from the completed review artifact, not from fresh partial analysis
- explain the issue with concrete file/doc references
- distinguish clearly between:
  - a true architectural mismatch
  - a wording/validation tightening
  - a non-issue that looked risky but is already covered by the plan

If the user agrees with a finding and asks to tighten the plan:

- patch the original plan directly
- update the review file checklist entry to `[x]`
- add a short resolution note in the review file pointing to the tightened section

## What To Check

At minimum, analyze the plan against:

- architecture docs and ADRs for the touched area
- current abstraction owners and boundary seams in code
- existing backlog intent or history notes that the plan claims to build on
- validation and end-state checks
- whether the plan reuses existing abstractions instead of creating parallel ones
- whether the plan leaves room for accidental parallel models, temporary shims, or owner drift

Prefer concrete owner checks such as:

- composition roots
- app/service boundaries
- shared modal or list/detail infrastructure
- shared contracts in `src/domain/`
- search execution vs shared semantic contract boundaries
- TUI-derived presentation vs durable semantic state

## Important Constraints

- Keep planning/review work uncommitted unless the user explicitly asks otherwise.
- Treat `scratch/plan-review/` artifacts as transient by default, not repo content to land.
- Do not start implementation just because the plan has issues.
- Do not rewrite the whole plan by default. First produce the review artifact.
- Do not let the chat answer substitute for the review file.
- Do not treat “I already found enough issues” as a reason to skip the rest of the pass.

## Good Defaults

- Prefer exact file and module references when known.
- Separate “finding” from “proposed correction”.
- Say whether an issue is architectural, boundary-related, validation-related, or only wording-related.
- Preserve the distinction between reviewing the plan and changing the plan.
- If the user likes the review process, keep using the checklist artifact as the shared source of truth during follow-up discussion.

## Reference

For a suggested review-file structure, use:

- `references/review-template.md`
