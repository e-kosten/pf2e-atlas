---
name: pf2e-derived-tag-exemplar-curation
description: Curate PF2E derived-tag exemplars into small, high-signal teaching sets. Use when Codex needs to prune oversized exemplar lists, improve positive and negative teaching examples, and send only uncertain exemplar calls to the workbench.
---

# PF2E Derived Tag Exemplar Curation

Use this skill when the task is primarily about exemplar quality rather than runtime tagging coverage.

This is a **live-authoring** skill:
- confident exemplar keep/drop decisions go directly into live exemplar files
- only uncertain exemplar decisions go into `src/tags/exemplar-reviews`
- if a touched record clearly needs assignment cleanup, fix that at the same time instead of ignoring it

## Required Output

For each touched record:
- decide whether it should remain a positive exemplar
- become or remain a negative exemplar
- be removed from exemplar use entirely

The output should be a smaller, stronger teaching set, not a large list of records that merely happen to fit the tag.

## Workflow

1. Start from oversized or weak exemplar sets.
   Use the current exemplar files and the existing exemplar cleanup workflow as discovery input.
2. Re-read the ontology boundary for the tag.
   Focus on the tag's description, applies/does-not-apply guidance, signals, and adjacent tags.
3. Judge exemplar quality, not just tag truth.
   A record can be a correct tagged member and still be a weak exemplar.
4. Keep only strong teaching examples.
   Favor records that are:
   - representative
   - easy to explain
   - not too adventure-specific or idiosyncratic
   - useful for distinguishing adjacent tags
5. Write confident exemplar edits directly.
   Update `src/tags/exemplars/<category>.ts`.
6. Route uncertain exemplar edits to review.
   Add or update entries in `src/tags/exemplar-reviews/<category>.ts`.
7. If a touched record has clearly wrong or missing explicit assignments, fix that too.
   Do not leave obvious tagging debt behind on records you are already reviewing.
8. Validate with `npm run build` and `npm test`.

## Important Rules

- Exemplars are explanatory ontology aids, not runtime tagging state.
- Negative exemplars should teach a meaningful nearby boundary, not just be random non-members.
- Keep exemplar sets short.
- Confident exemplar changes should bypass the workbench entirely.

