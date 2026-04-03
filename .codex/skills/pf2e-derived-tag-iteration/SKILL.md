---
name: pf2e-derived-tag-iteration
description: Compatibility entry point for PF2E derived-tag work when a request explicitly says iteration or mixes category expansion with follow-up calibration. Prefer pf2e-derived-tag-expansion for sparse-category ontology and coverage work, and prefer pf2e-derived-tag-refinement for focused tuning of existing tags.
---

# PF2E Derived Tag Iteration

Use this skill only as a router when the request is broad or explicitly says `iteration`.

Current derived-tag work should usually use one of these skills instead:
- `../pf2e-derived-tag-expansion/SKILL.md` for sparse categories, missing ontology, category audits, and coverage-moving passes
- `../pf2e-derived-tag-refinement/SKILL.md` for false positives, threshold tuning, narrow coverage extensions, and regression-focused cleanup

Choose `pf2e-derived-tag-expansion` when any of these are true:
- the category is still under-modeled at roughly 30% tagged coverage or below
- the category has too few tag families or tags to describe the corpus well
- repeated untagged samples show missing retrieval concepts
- the user asks for broad coverage gains, new tag families, or a category pass

Choose `pf2e-derived-tag-refinement` when any of these are true:
- the ontology is already present and the user wants to tune it
- the task is mainly about false positives or false negatives in an existing tag family or related slice
- the user wants a focused but not microscopic batch
- the goal is to rebalance anchors, blockers, thresholds, or scoring across a small related set

If the request genuinely mixes both:
1. Run a category audit first.
2. Use the expansion workflow to define the missing category slice.
3. Use the refinement workflow for any follow-up precision cleanup after the expansion is in place.
