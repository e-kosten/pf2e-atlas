---
name: pf2e-tag-batch
description: Tag new or under-tagged PF2E records in bounded batches. Use when Codex needs to read larger record slices, write confident live assignments directly, update exemplars when obvious, and leave only ambiguous calls for review.
---

# PF2E Tag Batch

Use this skill for forward coverage work on records that are currently untagged or under-tagged.

This is a **live-authoring** skill:

- confident assignment decisions go directly into `src/tags/assignments`
- only ambiguous assignment decisions go into `src/tags/assignment-reviews`
- confident exemplar updates go directly into `src/tags/exemplars`
- only ambiguous exemplar updates go into `src/tags/exemplar-reviews`

## Required Output

For every touched record:

- decide the record's long-term explicit assignments across the ontology
- update exemplars when the teaching value is obvious
- leave only true ambiguity for review

## Workflow

1. Start from a bounded untagged or sparse-tag slice.
2. Read the relevant ontology before tagging.
3. For each touched record, do a full future-state tagging pass across the ontology.
4. Write confident live assignments directly to `src/tags/assignments`.
5. Route only ambiguous assignment calls to `src/tags/assignment-reviews`.
6. If the record is clearly a strong or weak teaching example, update exemplars too.
7. Capture ontology gaps explicitly instead of compensating with brittle heuristics.
8. Validate with `npm run build` and `cd scripts && npm test`.

## Important Rules

- Do not invent weak pseudo-rules just to force coverage.
- Do not leave obvious high-confidence work for the workbench.
- If a record is touched, settle its future-state tagging as completely as possible.
- Treat the workbench as an ambiguity resolver, not the main authoring path.
