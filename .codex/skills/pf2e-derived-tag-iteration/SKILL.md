---
name: pf2e-derived-tag-iteration
description: Iteratively expand and refine PF2E derived tags in the Pathfinder MCP repo. Use when Codex needs to propose new derived tags, extend tag coverage across creatures, equipment, hazards, spells, or afflictions, tighten or rebalance heuristic rules, improve scored evidence thresholds, or validate that rebuilt-corpus tagging stays precise and useful without collapsing into heuristic spaghetti.
---

# PF2E Derived Tag Iteration

Use this skill for work on the derived-tag layer in the Pathfinder MCP repo, especially `src/derived-tags.ts`, `tests/derived-tags.test.ts`, and `tests/pf2e-data.test.ts`.

Derived tags are not replacements for native PF2E traits. Keep the separation sharp:
- native `traits` are authoritative game taxonomy
- derived tags add retrieval-oriented function, context, polarity, or scene-fit signals that native traits do not express cleanly
- share tag values across categories when the retrieval meaning is genuinely the same, but keep heuristic rules category-scoped unless the evidence model is stable across categories

Repo-owned helper tooling for this skill:
- `npm run evaluate-derived-tags -- --tag <derived_tag> [--category <category>] [--subcategory <subcategory>]`
- Use it as an offline gap-discovery aid, not as a live tagging source of truth
- Review its suggestions manually, then convert repeated real patterns into explicit deterministic rules and tests

## Workflow
1. Read the current derived-tag surface first.
   Inspect:
   - `src/derived-tags.ts`
   - `tests/derived-tags.test.ts`
   - `tests/pf2e-data.test.ts`
   - `src/pf2e-data.ts` only when storage, summaries, or filter exposure matter
2. Identify the current gap type before editing.
   Classify it as one of:
   - missing ontology: useful tag family does not exist yet
   - sparse coverage: a useful tag exists but misses obvious records
   - false positives: records land in a bucket they should not
   - poor calibration: scoring or thresholds let weak evidence outrank stronger evidence
3. Propose tag additions conservatively.
   For each new tag, state:
   - category and family
   - why native traits are not enough
   - what the tag means in retrieval terms
   - what evidence should trigger it
   - what evidence should block it
4. Stop for an approval checkpoint before editing.
   After identifying the next batch of tag or heuristic changes, summarize:
   - the proposed new tags or rule changes
   - the expected coverage gain
   - the main precision risks
   Then ask a plain-text confirmation question and wait for the user before changing files.
   In normal mode, do this conversationally. Do not proceed directly from proposal to edits unless the user explicitly asked for immediate implementation.
5. Implement in the declarative rule table, not ad hoc branching.
   Prefer:
   - atomic tags over composite tags
   - strong and weak anchors over giant undifferentiated keyword lists
   - scored `anyOf` clauses plus thresholds over one-shot boolean triggers
   - name and trait evidence over description-only evidence
   - resolved linked-record evidence over brittle prose anchors when the indexing pipeline already maps references to canonical records
6. Add or update tests with every heuristic change.
   Cover three levels when relevant:
   - direct derivation unit tests in `tests/derived-tags.test.ts`
   - service-level behavior in `tests/pf2e-data.test.ts`
   - rebuilt-corpus sanity checks when the rule change is meaningful enough to affect many records
7. Run the offline gap evaluator when coverage is the question.
   Use `npm run evaluate-derived-tags -- --tag <derived_tag> ...` after identifying a likely sparse tag family or after a meaningful rebuild.
   Treat the output as a review queue:
   - likely false negatives to inspect manually
   - semantically nearby untagged records that may suggest a missing rule or a missing tag family
   - candidate records that should stay untagged, which help define blockers and negative gates
   When a tag has stable retrieval meaning across categories, explicitly check cross-category ontology transfer before inventing a near-duplicate tag. Use cross-category exemplar seeding to probe likely target categories, but treat it as ontology discovery rather than proof that the tag belongs there.
   Do not directly convert evaluator suggestions into tags without explicit rule evidence.
8. Spot-check for missing coverage, not just bad hits.
   Pull random samples of untagged canonical records from the category slices relevant to the current pass. Use them to answer:
   - are these records truly fine without derived tags
   - is a current tag missing obvious coverage
   - is there a useful new tag family emerging from repeated patterns
   Prefer sampling by category and subcategory so the ontology grows from real gaps in the corpus instead of only from remembered examples.
9. Validate in layers.
   Run focused tests first, then build, then full test suite. Rebuild the index when the heuristic change is large enough that live corpus behavior matters.
10. Summarize the outcome in retrieval terms.
   Report:
   - what tag families changed
   - what new coverage was added
   - what false-positive classes were reduced
   - what remains intentionally conservative

## Design Rules
- Keep derived tags composable. Prefer `nautical` plus native `undead` over `nautical_undead`.
- Do not mirror PF2E-native traits with aliases like `fiend_threat` unless the tag encodes a materially different retrieval meaning.
- Prefer false negatives over false positives.
- Keep categories and families as catalog metadata, not a second filter language.
- Use scoring to reward distinct evidence, not word volume.
- Cap the effect of weak text clues by requiring multiple weak signals or one strong signal.
- Preserve explainability. A future reader should be able to tell why a rule fires by reading the rule table.
- If a rule needs many exceptions, redesign the evidence model instead of piling on exclusions.

## Linked Reference Guidance
When derived-tag rules need to reason about actions, spells, or other compendium-linked support records, prefer resolved linked-record evidence over matching raw Foundry UUID strings in prose.

- If the indexing pass already resolves `@UUID[...]` references to canonical records, run tag derivation after that resolution step and feed the resolved links into the rule context.
- Author linked-record rule anchors using a friendly `pack:name` shorthand such as `actionspf2e:Impersonate` or `spells-srd:Illusory Disguise`.
- Match internally against the canonical resolved target, not against raw text, so rule behavior stays stable even if display text changes.
- Treat the Foundry document-type segment like `Item` as optional authoring noise unless the corpus later proves it carries disambiguating value.
- Leave a short code comment near the helper that normalizes linked-reference anchors explaining this choice, so the implementation can be revisited if PF2E/Foundry ever stops making `pack:name` a stable shorthand.

## Scoring Guidance
When refining heuristics, prefer this evidence order:
- trait evidence: strongest
- name evidence: next strongest
- description evidence: useful but weaker

Good scoring patterns:
- one strong anchor is enough
- two weak anchors can substitute for one strong anchor
- conflicting monster-family or polarity evidence subtracts confidence or blocks the tag entirely

Avoid:
- raw substring matching
- repeated token counting from long descriptions
- broad description anchors that can fire on incidental flavor text

## Tag Proposal Heuristics
Good derived tags usually encode one of these:
- equipment function or polarity not captured cleanly by native traits
- gear purpose or practical use case
- creature environment or scene-context
- creature scene-fit distinctions such as `profession_npc` or `scene_adjacent`
- hazard practical function such as alarms, restraint, or infiltration blocking
- spell practical use case such as disguise, mobility, scouting, or escape
- affliction downstream impact such as mobility impairment or mental degradation
- shared cross-category retrieval meaning such as `disguise`, when category-specific rules can emit the same tag value without changing what the tag means

Bad default candidates:
- aliases for native PF2E traits
- composite tags that can already be expressed by combining a derived tag with native traits
- highly interpretive encounter-design roles unless the evidence model is very strong

## Precision Triage
When corpus spot checks show bad results:
- first ask whether the tag should exist at all
- then ask whether the trigger should require stronger evidence
- then ask whether a negative gate or threshold is missing
- only after that consider adding more anchors

Typical fixes:
- false positive from one weak cue: raise threshold or split strong/weak anchors
- support item tagged offensive: strengthen negative polarity gates and require offense-specific anchors or traits
- scene-fit tag landing on monsters: add monster-family blockers or require stronger name evidence
- environment tag overfiring from general flavor text: restrict to token-safe anchors or promote the tag to score-based matching

## Spot Checking
Use `references/sanity-checks.md` for both regression checks and gap-discovery sampling.

After a meaningful heuristic change, always inspect:
- known false-positive counters
- named records that have failed before
- evaluator suggestions for the changed or newly added tag family when coverage expansion is part of the goal
- a random sample of untagged canonical records from the changed category
- a random sample from a previously supported category when the change risks cross-category regressions

Treat untagged samples as a forward-looking backlog. Do not force every record to have a derived tag, but when the same practical concept appears repeatedly in random samples, consider whether the ontology is missing a useful tag or the current rules are too narrow.
Treat evaluator output the same way: useful for discovery, but subordinate to deterministic rule design and explicit tests.

## Approval Checkpoint
Before editing files, pause and present the intended change set as a small batch.

Good approval-batch shape:
- 1-3 new tags or one narrow rule-calibration pass
- concrete examples of records expected to change
- one sentence on likely false-positive risk

Ask a direct confirmation question after the proposal, for example:
- `Proceed with this batch, or do you want to adjust the proposed tags first?`
- `Proceed with the rule calibration pass, or narrow the scope before I edit?`

This skill is meant to iterate carefully. Favor small approved batches over large speculative expansions.

## Validation
Use `references/sanity-checks.md` for the default validation commands and corpus spot checks.

Default validation sequence:
1. `npm test -- tests/derived-tags.test.ts tests/pf2e-data.test.ts`
2. `npm run build`
3. `npm test`
4. `npm run refresh-index` when the heuristic change materially affects live tagging
5. Run `npm run evaluate-derived-tags -- --tag <derived_tag> ...` when the change is about expanding coverage or checking likely false negatives
6. Run the SQLite sanity checks from the reference file
7. Review at least one random untagged sample for the changed category, plus one previously supported category when regression risk matters

## Output Expectations
When the user asks for iteration work, do not stop at proposing tags. Carry the change through:
- update rules and catalog metadata
- add tests
- validate
- summarize what changed in the live corpus or explain why a rebuild was skipped
