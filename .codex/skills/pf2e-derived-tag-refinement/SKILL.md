---
name: pf2e-derived-tag-refinement
description: Refine existing PF2E derived tags in the Pathfinder MCP repo. Use when Codex needs false-positive cleanup, threshold or blocker tuning, focused coverage extensions for related existing tags, scored-anchor calibration, regression-focused rule edits, or a medium-grain refinement batch rather than broader ontology expansion.
---

# PF2E Derived Tag Refinement

Use this skill for precision work on the derived-tag layer, especially [`src/tags/index.ts`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/src/tags/index.ts), [`tests/tags/derived-tags.test.ts`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/tests/tags/derived-tags.test.ts), and the service tests that exercise derived-tag filtering.

Use this mode when the ontology mostly exists and the problem is calibration across a related slice, not just a one-tag micro-fix.

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
   - `tests/tags/derived-tags.test.ts`
   - the relevant `tests/service/*.test.ts`
2. Classify the issue before editing.
   Use one of:
   - `false positives`
   - `sparse coverage`
   - `poor calibration`
3. Keep the batch narrow.
   Good default scope:
   - 2-5 related existing tags, or
   - one related-slice refinement pass across a coherent part of the category
   Do not default to one-tag micro-passes unless the regression is truly isolated.
4. Use the evaluator as a review queue.
   Target the specific tag under refinement and inspect likely false negatives or semantically adjacent misses.
5. Stop for an approval checkpoint before editing.
   Include:
   - the tag or rules being changed
   - expected record movement
   - the main precision risk
   Ask a direct confirmation question and wait unless the user explicitly asked for immediate implementation.
6. Implement in the declarative rule table.
   Prefer:
   - stronger anchors over larger keyword lists
   - negative gates before exception piles
   - score thresholds over brittle boolean logic
   - trait and name evidence ahead of broad description evidence
7. Add or update tests with every refinement pass.
   Cover direct derivation and at least one service-level behavior where the regression matters.
8. Validate in layers.
   Use:
   - `npm test -- tests/tags/derived-tags.test.ts tests/service/search-and-lookup.test.ts`
   - `npm run build`
   - `npm test`
   - `npm run refresh-index -- --reuse-embeddings` only when the change should materially affect live tagging
9. Summarize in precision terms.
   Report:
   - what false-positive or false-negative class changed
   - which tags were tuned
   - what tests and spot checks defend the change
   - what remains intentionally conservative

## Precision Rules

- Prefer false negatives over false positives.
- If a rule needs many exceptions, redesign the evidence model.
- One strong anchor should beat many weak incidental words.
- Do not mirror PF2E-native traits with derived aliases unless the retrieval meaning is materially different.
- Keep changes explainable to a future reader scanning the rule table.

## Approval Batch Shape

This skill is intentionally small-batch.

A good approval batch is:
- 2-5 related existing tags or one related-slice refinement pass
- concrete examples of records expected to change
- one sentence on likely false-positive risk
