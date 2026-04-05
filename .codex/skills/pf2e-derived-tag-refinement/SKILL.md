---
name: pf2e-derived-tag-refinement
description: Refine existing PF2E derived tags in the Pathfinder MCP repo. Use when Codex needs false-positive cleanup, threshold or blocker tuning, focused coverage extensions for related existing tags, scored-anchor calibration, regression-focused rule edits, or a medium-grain refinement batch rather than broader ontology expansion.
---

# PF2E Derived Tag Refinement

Use this skill for precision work on the derived-tag layer, especially [`src/tags/index.ts`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/src/tags/index.ts), category rule modules under [`src/tags/rules`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/src/tags/rules), category derivation tests under [`tests/tags`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/tests/tags), and the service tests that exercise derived-tag filtering.

Use this mode when the ontology mostly exists and the problem is calibration across a related slice, not just a one-tag micro-fix.

The default posture is regression-first precision control. When a tag overfires on noisy generic language, pin the false-positive class with real records before touching anchors, blockers, thresholds, or scoring.

## Use This Mode

Default to this skill when any of these are true:
- an existing tag is underfiring on obvious records
- an existing tag is overfiring on the wrong class of records
- thresholds, blockers, anchors, or scoring need rebalancing
- the user wants a small safe pass or a narrow follow-up adjustment
- the task is mostly regression control, not category expansion

## Workflow

1. Read the current rules and tests first.
   Focus on:
   - `src/tags/index.ts`
   - `tests/tags/derived-tags-*.test.ts`
   - the relevant `tests/service/*.test.ts`
2. Build a real-record regression set before editing rules.
   For false-positive work, identify the reusable bad class and at least one real canonical record that currently overfires because of noisy generic language.
   Prefer using existing regression seeds when they fit the slice:
   - `Crushing Ground`
   - `Imprisonment`
   - `Artevil Suspension`
   - `Blindpepper Bomb`
   - `Mycological Malady`
   - one troll lore paragraph for creature-setting noise
   If none fit, name the replacement records explicitly.
3. Classify the issue before editing.
   Use one of:
   - `false positives`
   - `sparse coverage`
   - `poor calibration`
4. Keep the batch narrow.
   Good default scope:
   - 2-5 related existing tags, or
   - one related-slice refinement pass across a coherent part of the category
   Do not default to one-tag micro-passes unless the regression is truly isolated.
5. Use the evaluator as a review queue.
   Target the specific tag under refinement and inspect likely false negatives, semantically adjacent misses, and records that suggest the same false-positive class.
   Before or after the rule edit when live movement matters, compare index snapshots with:
   - `npm run evaluate-derived-tag-movement -- --baseline-index-path /path/to/before.sqlite --category <category> --tags <tag1,tag2,...> --warn-category-drop-points <points> --warn-tag-drop-count <count> --warn-tag-drop-points <points> --sample-limit <n>`
   Use this to catch suspicious live-coverage drops before calling the pass done.
6. Stop for an approval checkpoint before editing.
   Include:
   - the tag or rules being changed
   - the false-positive or false-negative class being addressed
   - the conceptual calibration logic, including intended anchors, blockers, thresholds, and the expected boundary between positive and negative cases
   - the real regression records that will be pinned before rule edits
   - expected record movement
   - expected category-level coverage delta when the pass should move live coverage materially
   - the main precision risk
   Ask a direct confirmation question and wait unless the user explicitly asked for immediate implementation.
   In parallel workflows, no worker should implement rule or test edits before this approval is granted for that worker's slice.
7. Add or update regression tests before changing the rule logic.
   For false-positive work, add direct derivation negatives from real records in the relevant `tests/tags/derived-tags-*.test.ts` file first. Positive-only test growth is not enough.
   When the regression matters to search behavior, add or update at least one service-level check that defends the boundary.
8. Implement in the declarative rule table.
   Prefer:
   - stronger anchors over larger keyword lists
   - negative gates before exception piles
   - score thresholds over brittle boolean logic
   - trait and name evidence ahead of broad description evidence
9. Validate in layers.
   Use:
   - `npm test -- tests/tags/derived-tags-*.test.ts tests/service/search-and-lookup.test.ts`
   - `npm run build`
   - `npm test`
   - `npm run refresh-index -- --reuse-embeddings` when the change should materially affect live tagging or when you need to confirm category-level coverage movement
   - `npm run evaluate-derived-tag-movement -- --baseline-index-path /path/to/before.sqlite --category <category> --tags <tag1,tag2,...> --warn-category-drop-points <points> --warn-tag-drop-count <count> --warn-tag-drop-points <points> --sample-limit <n>` when a refinement pass could materially reduce live coverage
10. Summarize in precision terms.
   Report:
   - what false-positive or false-negative class changed
   - which tags were tuned
   - which real regression records now defend the boundary
   - before/after tagged-record counts and percentage-point coverage delta when the pass materially moved live records
   - or state explicitly that the pass was precision-only and category-level coverage movement was not the goal
   - what tests and spot checks defend the change
   - what remains intentionally conservative

## Precision Rules

- Prefer false negatives over false positives.
- For false-positive cleanup, real noisy records beat synthetic examples.
- If a rule needs many exceptions, redesign the evidence model.
- One strong anchor should beat many weak incidental words.
- Do not mirror PF2E-native traits with derived aliases unless the retrieval meaning is materially different.
- Keep changes explainable to a future reader scanning the rule table.
- Do not rely on positive-skewed tests alone when broad description evidence is involved.
- If a refinement pass materially drops category coverage or a touched tag collapses in the movement evaluator, stop and reassess instead of hand-waving the loss away.

## Approval Batch Shape

This skill is intentionally small-batch.

A good approval batch is:
- 2-5 related existing tags or one related-slice refinement pass
- at least one named real-record regression case for each noisy false-positive class being fixed
- concrete examples of records expected to change
- one sentence on likely false-positive risk

## Parallel Approval-First Mode

When refinement work is split across multiple slices:

- Use a parent-agent workflow.
- Workers may inspect rules, gather false-positive or false-negative evidence, and draft refinement proposals.
- Workers must stop before implementation and hand the parent agent:
  - the affected tags or rule slice
  - conceptual calibration logic
  - the real regression records to pin first
  - expected record movement
  - principal precision risks and boundary cases
- The parent agent presents those proposals to the user for approval.
- Only approved refinement slices should proceed to implementation.
- The parent agent remains the gate for final integration, merge back into the main workspace, and the final commit.
