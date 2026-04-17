---
name: pf2e-tag-legacy-migration
description: Migrate legacy PF2E derived-tag rule coverage into future-state authored rules, explicit assignments, or retirement. Use when Codex needs to start from a legacy-rule-driven cohort, do a full long-term tagging pass on each touched record, and decide what survives in the future system.
---

# PF2E Tag Legacy Migration

Use this skill when the starting point is a legacy rule, legacy-rule-derived cohort, or other old deterministic coverage that needs a future-state disposition.

This is not only a rule-audit skill. It owns the full transition from legacy coverage to future-state behavior.

This is a live-authoring skill:
- confident explicit assignments go directly into `src/tags/assignments`
- confident future authored rules go directly into `src/tags/authored-rules`
- confident exemplar updates go directly into `src/tags/exemplars` when touched records clearly improve or weaken teaching value
- only uncertain assignment calls go into `src/tags/assignment-reviews`
- only uncertain exemplar calls go into `src/tags/exemplar-reviews`

## Required Output

For every touched record:
- perform a full future-state tagging pass across the ontology, not just the original legacy tag
- decide the record's durable explicit assignment set
- decide whether the record has exemplar implications worth updating

For the legacy rule slice itself:
- decide whether it should be:
  - recreated as a future authored rule
  - replaced by explicit assignments
  - dropped as too noisy or too weak

## Workflow

1. Start from all records currently touched by the target legacy rule or legacy-rule slice.
2. Read the ontology, current legacy-rule behavior, and nearby future-state authored rules first.
3. Build a real-record migration set before changing anything.
4. For each touched record, do a full future-state tagging pass.
   Do not only patch the one legacy-rule tag.
5. Write confident assignment decisions directly to `src/tags/assignments`.
6. Route uncertain assignment calls to `src/tags/assignment-reviews`.
7. Decide the rule disposition.
   - if a safe deterministic future authored rule clearly exists, write it directly
   - if the cohort is too fuzzy for a durable rule, replace it with explicit assignments only
   - if the right answer is that the legacy concept should disappear, retire it
8. If touched records are also useful or misleading exemplars, update exemplars at the same time.
9. Validate with focused tests first, then `npm run build` and `npm test`.

## Important Rules

- The legacy rule is the discovery entrypoint, not the scope boundary.
- Every touched record gets a full long-term treatment, not a narrow rule-preservation pass.
- Do not preserve a noisy rule out of historical preference.
- Do not defer obvious high-confidence assignments or exemplar decisions to review.
- If no durable future-state rule exists, prefer explicit assignments or retirement over force-fitting rule logic.
