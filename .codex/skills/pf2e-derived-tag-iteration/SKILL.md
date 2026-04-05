---
name: pf2e-derived-tag-iteration
description: Compatibility entry point for PF2E derived-tag work when a request explicitly says iteration or mixes category expansion with follow-up calibration. Prefer pf2e-derived-tag-expansion for sparse-category ontology and coverage work, and prefer pf2e-derived-tag-refinement for focused tuning of existing tags.
---

# PF2E Derived Tag Iteration

Use this skill only as a router when the request is broad or explicitly says `iteration`.

Route with a regression-first bias. If any slice risks overfiring on generic or noisy language, require real-record regression capture before rule changes, not just a later validation pass.

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

Use these as preferred regression seeds when they fit the slice:
- `Crushing Ground`
- `Imprisonment`
- `Artevil Suspension`
- `Blindpepper Bomb`
- `Mycological Malady`
- one troll lore paragraph for creature-setting noise

If the request genuinely mixes both:
1. Run a category audit first.
2. Run deterministic discovery on the likely slice before proposing rules:
   - `npm run analyze-derived-tag-evidence -- --category <category> ...`
   - `npm run discover-ruleable-cohorts -- --category <category> ...`
3. Identify any likely false-positive classes and the real records that should be pinned as regressions before implementation.
4. Use the expansion workflow to define the missing category slice.
5. Use the refinement workflow for any follow-up precision cleanup after the expansion is in place.
6. Preserve one baseline coverage snapshot before expansion, one post-expansion coverage check, and one final summary that separates ontology gain from later precision cleanup.
   Use `npm run evaluate-derived-tag-movement -- --baseline-index-path /path/to/before.sqlite ...` as the default before/after check.
   For expansion-oriented slices, prefer `--warn-category-gain-below-points`, `--warn-tag-gain-below-count`, and `--sample-limit`.
   For refinement-oriented slices, prefer `--warn-category-drop-points`, `--warn-tag-drop-count`, and `--warn-tag-drop-points`.
7. In parallel workflows, keep all slices in proposal mode until the user has approved the proposed tags, conceptual rule logic, deterministic evidence summary, and regression records for each slice that will proceed.

Do not finish an iteration pass that was supposed to improve coverage without a quantified movement summary. Carry through the routed workflow's coverage reporting expectations:
- tagged-record counts before and after
- tagged coverage percentage before and after
- tagged-record delta and percentage-point gain when coverage was the goal
- movement-evaluator warnings or explicit note that the touched tags passed the chosen gain/drop thresholds
- explicit note when a later refinement step was precision-only rather than coverage-moving
- explicit note on which real-record regression cases were added to prevent noisy false positives

## Parallel Approval-First Mode

When iteration work is broad enough to split:

- Use a parent-agent workflow.
- Workers may audit and prepare proposals for their slice, but they should not implement before approval.
- The parent agent should collect per-slice proposals and present them to the user first.
- Each proposal should include:
  - the slice
  - proposed tags or refinement targets
  - conceptual rule logic
  - the real regression records to pin first when noisy-language risk exists
  - expected movement
  - major risks
- Only after approval should the parent agent let workers implement their slice.
- The parent agent remains the gate for final integration, merge back into the main workspace, and the final commit.
