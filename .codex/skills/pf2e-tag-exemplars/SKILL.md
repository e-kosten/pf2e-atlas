---
name: pf2e-tag-exemplars
description: Curate PF2E derived-tag exemplars into small, high-signal teaching sets. Use when Codex needs to prune weak exemplar lists, improve positive and negative examples, and send only uncertain exemplar calls to review.
---

# PF2E Tag Exemplars

Use this skill when the task is primarily about exemplar quality rather than runtime tagging coverage.

This is a **live-authoring** skill:

- confident exemplar keep/drop decisions go directly into `src/tags/exemplars`
- only uncertain exemplar decisions go into `src/tags/exemplar-reviews`
- if a touched record clearly needs assignment cleanup, fix that too instead of ignoring it

## Required Output

For each touched record:

- decide whether it should remain a positive exemplar
- become or remain a negative exemplar
- be removed from exemplar use entirely

The output should be a smaller, stronger teaching set, not a large list of records that merely happen to fit the tag.

## Workflow

1. Start from oversized or weak exemplar sets.
2. Re-read the ontology boundary for the tag.
3. Judge exemplar quality, not just tag truth.
4. Keep only strong teaching examples that are representative and easy to explain.
5. Write confident exemplar edits directly to `src/tags/exemplars/<category>.ts`.
6. Route uncertain exemplar edits to `src/tags/exemplar-reviews/<category>.ts`.
7. If a touched record has clearly wrong or missing explicit assignments, fix that too.
8. Validate with `npm run build` and `cd scripts && npm test`.

## Important Rules

- Exemplars are explanatory ontology aids, not runtime tagging state.
- Negative exemplars should teach a meaningful nearby boundary.
- Keep exemplar sets short.
- Confident exemplar changes should bypass the workbench entirely.
