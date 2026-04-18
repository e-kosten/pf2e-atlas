---
name: pf2e-tag-seeds
description: Review legacy seed-carried PF2E records and convert them into future-state assignments and exemplars. Use when Codex needs to do a full tagging pass on records discovered from old seed membership.
---

# PF2E Tag Seeds

Use this skill when a task starts from legacy seed membership but the desired outcome is future-state authored tagging.

This is a **live-authoring** skill:

- confident assignments go directly into `src/tags/assignments`
- confident exemplar changes go directly into `src/tags/exemplars`
- only uncertain assignment calls go into `src/tags/assignment-reviews`
- only uncertain exemplar calls go into `src/tags/exemplar-reviews`

## Required Output

For every touched record:

- perform a full future-state tagging pass across the ontology, not just the seeded tag
- decide the record's long-term explicit assignments
- decide whether the record should remain a positive exemplar, become a negative exemplar, or be dropped entirely

Once a record is touched in this pass, it should be as complete as possible for long-term migration.

## Workflow

1. Discover the legacy seed slice.
2. Read the relevant ontology first.
3. For each touched record, do a full record pass across the ontology.
4. Write confident assignments directly to `src/tags/assignments`.
5. Route uncertain assignments to `src/tags/assignment-reviews`.
6. Evaluate exemplar quality separately from tag truth.
7. Write confident exemplar outcomes directly to `src/tags/exemplars/<category>.ts`.
8. Route uncertain exemplar outcomes to `src/tags/exemplar-reviews/<category>.ts`.
9. Note ontology gaps explicitly instead of guessing around them.
10. Validate with `npm run build` and `npm test`.

## Important Rules

- The seeded tag is only the discovery entrypoint, not the scope boundary.
- Do not use exemplars as disguised membership lists.
- Do not leave obvious high-confidence decisions for manual review.
- If a record is a poor teaching example, remove it from exemplars even if the tag still applies.
