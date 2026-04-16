---
name: pf2e-derived-tag-seed-migration
description: Migrate legacy seed-derived PF2E records into future-state authored assignments and curated exemplars. Use when Codex needs to review records carried by old seed membership, write confident live tagging decisions directly, and leave only uncertain assignment or exemplar decisions for the workbench.
---

# PF2E Derived Tag Seed Migration

Use this skill when a task starts from legacy seed membership but the desired outcome is future-state authored tagging.

This is a **live-authoring** skill:
- edit the real authored files directly
- write confident assignment decisions as `auto_applied`
- write confident exemplar changes directly into live exemplar files
- leave only uncertain assignment decisions as `needs_review`
- leave only uncertain exemplar decisions in `src/tags/exemplar-reviews`

Do not use the migration workbench as the default output sink for confident work. The workbench is only for the unresolved subset.

## Required Output

For every touched record:
- perform a full future-state tagging pass across the ontology, not just the seeded tag
- decide the record's long-term explicit assignments
- decide whether the record should remain a positive exemplar, become a negative exemplar, or be dropped as an exemplar entirely

Once a record is touched in this pass, it should be as complete as possible for long-term migration.

## Workflow

1. Discover the legacy seed slice.
   Use the existing legacy seed migration surface to identify the records currently carried forward from old seeds.
2. Read the ontology first.
   Review the category ontology and adjacent tags before deciding assignments.
3. For each touched record, do a full record pass.
   Decide all clear long-term assignments across the ontology, not only the original seed tag.
4. Write confident assignments directly.
   - add or update authored assignment review entries
   - use `auto_applied` for confident decisions
   - sync them into live `applied` / `excluded`
5. Route uncertain assignments to review.
   - write them into authored assignment `review`
   - use `needs_review`
   - do not sync them into live `applied` / `excluded`
6. Evaluate exemplar quality separately from tagging.
   Exemplar decisions are about whether the record teaches the tag boundary well, not merely whether the tag applies.
7. Write confident exemplar outcomes directly.
   Update `src/tags/exemplars/<category>.ts` when the exemplar call is clear.
8. Route uncertain exemplar outcomes to review.
   Add or update entries in `src/tags/exemplar-reviews/<category>.ts`.
9. Note ontology gaps explicitly.
   If the ontology lacks a needed tag or has unclear boundaries, capture that as a short note instead of guessing around it.
10. Validate.
   Run `npm run build` and `npm test` when the batch is complete.

## Important Rules

- A seeded tag is only the selection entrypoint, not the scope limit.
- Do not leave obvious high-confidence decisions for manual review.
- Do not use exemplars as disguised membership lists.
- If a record is a poor teaching example, remove it from exemplars even if the tag still applies.
- Keep exemplar-review entries only for true ambiguity.

