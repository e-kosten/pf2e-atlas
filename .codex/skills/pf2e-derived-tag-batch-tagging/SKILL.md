---
name: pf2e-derived-tag-batch-tagging
description: Add future-state PF2E derived-tag coverage on untagged or under-tagged records. Use when Codex needs to read larger record batches, write confident explicit assignments directly, and surface ontology gaps while leaving only ambiguous decisions for human review.
---

# PF2E Derived Tag Batch Tagging

Use this skill for forward coverage work on records that are currently untagged or under-tagged.

This is a **live-authoring** skill:
- confident assignment decisions are written directly into authored assignment files
- only ambiguous assignment decisions go into `src/tags/assignment-reviews`
- ontology gaps are surfaced as explicit notes instead of being papered over with weak heuristics

## Required Output

For every touched record:
- decide the record's long-term explicit assignments across the ontology
- write confident decisions directly
- leave only true ambiguity for review

While doing so:
- identify missing tags
- identify unclear boundaries or weak ontology guidance
- suggest ontology improvements when needed

## Workflow

1. Start from untagged or sparse-tag records in a bounded batch.
2. Read the relevant ontology slice before tagging.
3. For each touched record, do a full future-state pass across the ontology.
4. Write confident assignment decisions directly.
   Update `src/tags/assignments` directly with durable provenance.
5. Route only ambiguous assignment decisions to review.
   Write them into `src/tags/assignment-reviews`.
6. If a touched record is clearly a strong or weak teaching example for a tag, update exemplars too.
   Confident exemplar decisions go directly to live exemplars; uncertain ones go to exemplar reviews.
7. Capture ontology gaps explicitly instead of compensating with brittle logic.
8. Validate with `npm run build` and `npm test`.

## Important Rules

- Do not invent brittle pseudo-rules just to force coverage.
- Do not leave obvious high-confidence work for the workbench.
- If a record is touched, settle its future-state tagging as completely as possible.
- Treat the workbench as a small ambiguity resolver, not as the main authoring path.
