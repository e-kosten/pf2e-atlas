---
name: pf2e-derived-tag-legacy-rule-migration
description: Migrate PF2E records currently tagged by legacy rules into future-state explicit assignments and future authored rules. Use when Codex needs to review every record touched by an old rule, do a full long-term tagging pass on each one, and decide whether the legacy rule should be recreated, replaced, or dropped.
---

# PF2E Derived Tag Legacy Rule Migration

Use this skill when starting from records currently tagged by a legacy rule or legacy-rule-derived tag.

This is **not** only a rule-audit skill. It is also a full record-by-record migration skill.

This is a **live-authoring** skill:
- confident explicit assignments are written directly into authored assignment files
- confident future authored rules are written directly into `src/tags/authored-rules`
- only uncertain assignments or uncertain rule disposition should go to review

## Required Output

For every touched record:
- perform a full future-state tagging pass across the ontology
- decide the record's complete explicit assignment set, not just the original legacy-rule tag
- decide whether the record has any exemplar implications worth updating

For the legacy rule itself:
- decide whether it should be:
  - recreated as a future authored rule
  - replaced by explicit assignments
  - dropped as too noisy

## Workflow

1. Start from all records currently tagged by the target legacy rule or rule slice.
2. Read the ontology and the current legacy-rule behavior first.
3. For each touched record, do a full future-state tagging pass.
   Do not only patch the one legacy-rule tag.
4. Write confident assignment decisions directly.
   Use `auto_applied` for confident decisions and sync live `applied` / `excluded`.
5. Route uncertain assignment calls to review.
   Use assignment `needs_review`.
6. Decide the rule disposition.
   - if a safe deterministic future authored rule clearly exists, write it directly
   - if the rule is too noisy, replace it with explicit assignments only
   - if rule disposition is ambiguous, leave that part for review
7. If touched records are also useful or misleading exemplars, update exemplars at the same time.
8. Validate with `npm run build` and `npm test`.

## Important Rules

- The legacy rule is only the discovery entrypoint, not the scope boundary.
- Every touched record gets the same full long-term treatment as a seed-migration record.
- Do not preserve a noisy rule out of historical preference.
- Do not defer obvious high-confidence assignments to the workbench.

