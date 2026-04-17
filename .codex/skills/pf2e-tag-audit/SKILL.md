---
name: pf2e-tag-audit
description: Audit suspicious PF2E tag density and overfit behavior. Use when Codex needs to inspect densely tagged records, identify false-positive classes, and hand off a regression-ready brief for rule or assignment cleanup.
---

# PF2E Tag Audit

Use this skill when the first sign of trouble is suspicious tag density rather than one clearly broken rule.

Dense tagging is not proof of a bug. It is a review queue for finding:
- overly broad weak anchors
- same-family tag explosions
- incidental source phrases that trigger setting or motif tags
- records whose tag set no longer matches the source text

## Workflow

1. Use the current local index.
   If derived-tag behavior changed recently, rebuild first with `npm run refresh-index -- --reuse-embeddings`.
2. Generate a dense-tag review queue with:
   `node --import tsx/esm .codex/skills/pf2e-tag-audit/scripts/dense-tag-report.ts --limit 30`
3. Treat the output as triage, not verdict.
4. Read the source record before touching rules or assignments.
5. Turn suspicious records into reusable false-positive classes.
6. Identify whether the likely fix belongs in:
   - `$pf2e-tag-rules` for deterministic rule cleanup
   - `$pf2e-tag-legacy-migration` when the suspicious cohort is coming from legacy rule coverage that needs full future-state disposition
   - `$pf2e-tag-batch` for explicit assignment cleanup on fuzzy concepts
7. Produce a short remediation brief with the suspicious records, suspect tags, likely cause, and recommended next skill.

## Important Rules

- Do not fix a single named record in isolation.
- Prefer naming one reusable false-positive class and one or two real regression records.
- The useful output from this skill is a cleanup brief, not a patch by default.
