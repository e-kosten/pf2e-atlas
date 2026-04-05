---
name: pf2e-derived-tag-expansion
description: Expand PF2E derived-tag ontology and coverage in the Pathfinder MCP repo. Use when Codex needs category audits, missing tag-family discovery, sparse-category passes, broader coverage gains across hazards, spells, afflictions, equipment, or creatures, or a meaningful category slice rather than a narrow calibration pass.
---

# PF2E Derived Tag Expansion

Use this skill for sparse-category work on the derived-tag layer, especially [`src/tags/index.ts`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/src/tags/index.ts), category rule modules under [`src/tags/rules`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/src/tags/rules), category derivation tests under [`tests/tags`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/tests/tags), and service-level search tests under [`tests/service`](/Users/ekosten/projects/pathfinder-mcp/pathfinder-2e-foundry-mcp/tests/service).

Derived tags are retrieval-oriented overlays, not aliases for native PF2E traits. Add tags only when native traits do not already express the practical retrieval meaning cleanly.

Expansion is still precision work. When a new tag or anchor could be triggered by broad generic text, design the negative boundary up front and pin it with a real noisy record before trusting the rule.

## Use This Mode

Default to this skill when any of these are true:
- the category has fewer than 30% tagged canonical records
- the category has fewer than 15 catalog tags total
- the category has only 1-2 live tag families
- random untagged samples keep surfacing repeated concepts with no ontology support
- the user asks for broader coverage, sparse categories, new tag families, or a category-wide audit

## Workflow

0. If the task starts at category selection rather than a fixed category, sketch potential tags for each plausible category before proposing one.
   Collect a lightweight matrix per candidate category:
   - current coverage snapshot
   - repeated untagged concepts
   - likely families and tags that could form one meaningful batch
   - rough expected record movement
   - obvious precision risks or noisy boundary classes
   Use this pass to compare category slices, not to lock rules yet. Do not recommend a category purely because its coverage is low; recommend it because it appears to support the strongest meaningful batch.
1. Audit the category before proposing tags.
   Collect:
   - canonical record count
   - tagged-record count
   - tagged coverage percentage
   - distinct live tags firing in the category
   - catalog family count and tag count for the category
   - representative untagged samples by category and subcategory
2. Classify the gap.
   Use one of:
   - `ontology deficit`
   - `sparse rule coverage`
   - `mixed deficit`
3. Look for practical retrieval concepts, not lore themes.
   Good candidates are functions or downstream consequences that recur across many records and are not already captured by native traits.
4. Prefer category slices over micro-passes.
   A good batch here is usually:
   - 1-2 new families and 3-8 tags, or
   - one existing family plus multiple meaningful rule extensions that should move dozens of records
   - one medium-size category pass that materially improves an under-modeled category even if no new family is needed
   When comparing multiple categories, prefer the category whose likely batch looks both meaningful and internally coherent, even if another category has slightly lower coverage.
5. Use semantic discovery and the gap evaluator as discovery support, not truth.
   - If the concept already exists as a derived tag, run `npm run evaluate-derived-tags -- --tag <derived_tag> ...`.
   - If the concept does not exist yet, seed it with exemplars and run `npm run discover-derived-tag-candidates -- --category <category> --name <record> ...`.
   - Preserve a baseline index snapshot before expansion and compare it after the rebuild with:
     `npm run evaluate-derived-tag-movement -- --baseline-index-path /path/to/before.sqlite --category <category> --tags <tag1,tag2,...> --warn-category-gain-below-points <points> --warn-tag-gain-below-count <count> --sample-limit <n>`
   - Use the movement evaluator to ask whether the expansion actually moved enough live records to justify the added rule complexity, and inspect gained/lost record samples for the touched tags.
   - Use the semantic output to identify likely candidates, repeated traits, repeated phrases, and false-positive classes. Do not treat embedding proximity as a direct tagging decision.
   - When a proposed tag depends on broad description evidence, identify at least one real canonical record that uses similar generic language but should stay negative.
   - Good regression seeds when they match the slice include `Crushing Ground`, `Imprisonment`, `Artevil Suspension`, `Blindpepper Bomb`, `Mycological Malady`, and one troll lore paragraph for creature-setting noise.
6. Stop for an approval checkpoint before editing.
   Include:
   - current coverage snapshot
   - proposed families and tags
   - the conceptual rule logic for each proposed tag, including likely anchors, blockers, thresholds, and boundary cases
   - expected tagged-record gain
   - expected percentage-point coverage gain
   - concrete example records expected to move
   - the real noisy boundary records that should remain negative
   - main precision risks
   When semantic discovery was used, also include:
   - exemplar set
   - top semantic candidates
   - repeated evidence terms or phrases
   - contrast records that show likely boundary cases
   Ask a direct confirmation question and wait unless the user explicitly asked for immediate implementation.
   In parallel workflows, no worker should implement rule, catalog, or test edits before this approval is granted for that worker's slice.
7. Implement in the declarative rule table.
   Prefer:
   - atomic tags
   - strong and weak anchors with thresholds
   - trait and name evidence over broad description-only evidence
   - linked-record evidence when available
8. Add tests with every expansion batch.
   Cover:
   - direct derivation in `tests/tags/derived-tags-*.test.ts`
   - service-level filtering and lookup behavior in the relevant `tests/service/*.test.ts`
   - rebuilt-corpus sanity checks when the pass should materially change live coverage
   - at least one real-record negative regression whenever a new tag or anchor could overfire on generic language
   Pin the regression before or alongside the rule edit; do not rely on positive exemplars alone for broad anchors.
9. Validate in layers.
   Start focused, then widen:
   - `npm test -- tests/tags/derived-tags-*.test.ts tests/service/search-and-lookup.test.ts`
   - `npm run build`
   - `npm test`
   - `npm run refresh-index -- --reuse-embeddings` when live tagging should change materially
   - `npm run evaluate-derived-tag-movement -- --baseline-index-path /path/to/before.sqlite --category <category> --tags <tag1,tag2,...> --warn-category-gain-below-points <points> --warn-tag-gain-below-count <count> --sample-limit <n>` when the pass is supposed to improve live coverage
10. Summarize at category level.
   Report:
   - before/after tagged-record counts
   - before/after tagged coverage percentage
   - tagged-record delta
   - percentage-point coverage gain
   - before/after distinct live tags firing
   - live-tag delta
   - example newly covered records by tag
   - false-positive classes checked
   - what remains intentionally unmodeled
   If the canonical denominator changed between the baseline audit and rebuilt corpus, say so explicitly and report both denominators rather than implying a fixed total.
   Include one compact coverage line in the final report, for example: `36/718 -> 100/720 tagged hazards, +64 records, +8.9 percentage points`.

## Semantic Discovery Examples

Use semantic discovery when you have a retrieval concept but no existing tag yet.

- Creature example:
  `npm run discover-derived-tag-candidates -- --category creature --name "Ghost Commoner" --name "Ghost Pirate Captain" --name "Cairn Wight"`
- Non-creature example:
  `npm run discover-derived-tag-candidates -- --category equipment --subcategory gear --name "Masquerade Scarf" --name "Quick-Change Outfit"`

Read the output as an evidence-mining pass. The next step is still to design explainable rule anchors, blockers, and thresholds in the declarative tag table.

## Design Rules

- Prefer meaningful category movement over polishing a tiny existing slice.
- Do not add tags only to drive the untagged count toward zero.
- If repeated untagged samples point to the same concept, treat that as ontology pressure.
- Keep derived tags composable and category-scoped unless the retrieval meaning is truly shared.
- Prefer explainable rule tables over ad hoc branching or giant keyword dumps.
- Every broad anchor should have a named noisy boundary case.
- Favor false negatives over false positives, but do not let that become an excuse to avoid needed ontology growth.
- If an expansion pass gains only a handful of live records, treat that as a possible over-specific rule set and inspect the gained-record samples before presenting the pass as successful.

## Approval Batch Shape

For this skill, the default batch is not `1-3` tags.

A good approval batch is:
- one sparse category slice
- 1-2 families and 3-8 tags, or a similarly meaningful expansion pass
- enough expected movement that the category coverage numbers should visibly change

When the task begins before category selection:
- first show the candidate-category matrix with likely tags and expected movement
- then recommend the best category slice
- only after that recommendation is accepted should the approval checkpoint drill into final per-tag rule logic for that category

## Parallel Approval-First Mode

When the work is split across multiple slices:

- Use a parent-agent workflow.
- Workers may audit, sample records, run evaluator/discovery commands, and draft proposals.
- Workers must stop before implementation and hand the parent agent:
  - proposed tags
  - conceptual rule logic
  - the noisy boundary records to pin as regressions
  - likely anchors, blockers, thresholds, and false-positive risks
  - representative records expected to move
- The parent agent presents those proposals to the user for approval.
- Only approved slices should proceed to implementation.
- Rejected slices should be dropped without further implementation work.
- The parent agent remains the gate for final integration, merge back into the main workspace, and the final commit.
