---
name: pf2e-tag-rules
description: Build, replace, or refine future-state deterministic PF2E tag rules. Use when Codex needs to tune noisy authored rules, expand safe rule coverage, decide assignment takeovers for current deterministic logic, and implement regression-first rule maintenance.
---

# PF2E Tag Rules

Use this skill for in-place deterministic rule work in the future-state derived-tag layer.

This includes:
- refining noisy or underfiring future authored rules
- expanding deterministic rule coverage when a safe, explainable future-state rule clearly exists
- replacing an existing future-state rule with explicit assignments when it proves too noisy

Use another skill instead when:
- the work starts from legacy rule coverage or a legacy-rule-driven cohort: use `$pf2e-tag-legacy-migration`
- the work is mainly manual assignment coverage on untagged records: use `$pf2e-tag-batch`
- the work starts from legacy seed membership: use `$pf2e-tag-seeds`
- the work is mainly exemplar quality: use `$pf2e-tag-exemplars`
- the work is mainly finding suspicious dense-tag false positives: use `$pf2e-tag-audit`

## Required Output

For every touched record:
- decide whether its tags should remain rule-driven or move to explicit assignments
- add or update regression cases before broad rule edits

For the rule slice itself:
- decide whether it should be kept as a deterministic authored rule
- narrowed or redesigned
- replaced by explicit assignments when it is too noisy

## Workflow

1. Read the ontology, current authored rules, and relevant tests first.
2. Build a real-record regression set before changing rule behavior.
3. Use discovery and evidence tools to understand the actual cohort, not just one record.
4. Decide whether the slice is:
   - safe deterministic rule work
   - mixed rule plus assignment takeover
   - assignment-only because the current rule is too fuzzy
5. Write confident live assignments directly to `src/tags/assignments`.
6. Route uncertain assignment calls to `src/tags/assignment-reviews`.
7. Add or refine future deterministic logic in `src/tags/authored-rules` only when the rule is explainable and durable.
8. Validate with focused tests first, then `npm run build` and `npm test`.

## Important Rules

- Do not preserve a noisy rule out of historical preference.
- Prefer strong anchors, blockers, and thresholds over broad weak heuristics.
- Pin false-positive classes with real records before tuning rules.
- If a rule needs many exceptions, replace it with assignments instead.
