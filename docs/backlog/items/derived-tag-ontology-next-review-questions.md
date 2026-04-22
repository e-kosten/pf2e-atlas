# Derived-Tag Ontology Next Review Questions

Status: proposed
Priority: soon
Owner: unassigned
Last reviewed: 2026-04-21

## Problem

The long-term backlog item for derived-tag ontology work captures the current model and the major resolved decisions, but the next immediate review questions are still spread across scratch artifacts and prior discussion.

That makes it harder to resume the ontology pass after a long gap, because the remaining work is not just "review medium-confidence rows" in general. There are a few specific families where the next decision is clear, bounded, and worth preserving in one restart-friendly note.

## Desired Outcome

Preserve the next immediate ontology review questions in one durable tracked note so a later session can restart from the actual pending decisions instead of reconstructing them from chat history.

This note should make it easy to answer:

- which families should be reviewed next
- what the fault line is inside each family
- what decisions are still open
- what provisional direction already seems most likely

## Constraints

- Do not reopen decisions already marked resolved in the broader ontology backlog item unless new evidence appears.
- Keep browse structure and canonical semantics separate. A family may remain useful for browsing even if its rows split across multiple canonical facet kinds.
- Keep one canonical label per concept. Do not solve awkward cross-category semantics by allowing projection-local naming drift.
- Prefer reusing existing facet kinds and operational shapes when they fit cleanly instead of inventing new kinds prematurely.
- Composite tags should remain canonical aggregate concepts only when they are meaningful filter targets.

## Notes

### Recommended Review Order

1. `spell/support`
2. `spell/control`
3. `hazard/countermeasure_profile`
4. `hazard/environmental_danger`
5. `equipment/anti_magic`

### Open Review Questions

#### `spell/support`

Why this needs review:

- It is still one of the largest remaining medium-confidence families.
- It mixes remediation, protection, survivability, and enhancement concepts.
- It smells similar to the equipment `function` family that was already split successfully.

What to decide:

- Which rows are true operational support/remediation concepts.
- Which rows are better treated as descriptive capability concepts.
- Whether some enhancement-style rows should remain grouped only for browse purposes while splitting semantically.

Known fault lines:

- `anti_*` rows are likely straightforward operational remediation concepts.
- `protective_ward` and `death_prevention` likely remain operational, but should be validated explicitly.
- Enhancement-style rows such as `initiative_support`, `quickened_support`, and possibly some resistance/protection rows may want capability treatment instead of being treated as answer-path operations.

#### `spell/control`

Why this needs review:

- The family is clearly useful for browsing, but canonically it may be too heterogeneous to treat as one operational pattern.
- It spans denial, battlefield shaping, countermagic, and pressure concepts.

What to decide:

- Whether `control` should remain only a browse family while its members normalize into sharper operational sub-patterns.
- Whether some current mappings are still too broad and need more specific operational names.

Known fault lines:

- denial: `action_denial`, `mobility_denial`, `silencing`
- battlefield shaping: `barrier_creation`, `battlefield_disruption`, `line_of_sight_control`
- anti-magic: `countermagic`
- pressure/manipulation: `fear_pressure`

The likely direction is to keep the browse family and continue sharpening the canonical operational shapes underneath it rather than forcing one shared operational meaning for the whole family.

#### `hazard/countermeasure_profile`

Why this needs review:

- This is one of the strongest cross-category bridges between problems and answer-paths.
- If it is clean, hazard retrieval will align much better with spell and equipment answer concepts.

What to decide:

- Whether the current countermeasure concepts align cleanly with the approved operational vocabulary.
- Whether any rows are still too broad or hide multiple answer-paths.
- Whether the cross-category naming is fully consistent now that discovery and answer operations are being normalized more strictly.

Rows worth validating carefully:

- `appeasement_countermeasure`
- `contamination_cleanup_countermeasure`
- `dispel_countermeasure`
- `exorcism_countermeasure`
- `physical_disarm`
- `procedural_bypass`
- `quarantine_containment_countermeasure`
- `source_cleanup_countermeasure`

The likely direction is validation and naming cleanup, not major model redesign.

#### `hazard/environmental_danger`

Why this needs review:

- A lot of hazard-side danger rows currently normalize into broad effect or application patterns.
- That is often correct, but a few rows may want sharper treatment as fields, persistent states, or environmental conditions instead of generic application concepts.

What to decide:

- Which rows are genuinely "applies this harmful thing."
- Which rows are better understood as hazard fields, contamination zones, or environmental-state concepts.

Known fault lines:

- likely straightforward: many elemental or direct damage hazards
- likely edge cases: `contamination_hazard`, `cursefield_hazard`

The likely direction is to keep many rows in existing operational patterns while tightening the few field/state edge cases.

#### `equipment/anti_magic`

Why this needs review:

- It is small, but it tests an important semantic boundary.
- `countermagic` and `magic_protection` are near each other in browse space while potentially belonging to different canonical shapes.

What to decide:

- Whether `countermagic` is a clear operational answer-path concept.
- Whether `magic_protection` is better understood as operational protection or as a descriptive capability/protection concept.
- Whether this family should eventually split semantically even if it remains a useful browse family.

Likely direction:

- `countermagic` remains operational.
- `magic_protection` deserves a deeper check because it may read more like persistent protective capability than direct answer-path action.

### Current Restart Context

The broader ontology work already resolved the following nearby issues, so they should not be reopened casually during this next pass:

- `regional_setting` is resolved as a setting-first space
- `bound_object` and `ontology_cluster` now map into `creature_family`
- `outbreak_response` was dropped in favor of `outbreak_containment`
- `epidemiological_profile` is a heterogeneous browse family under `disease_model`
- exposed composite tags are canonical `AggregateConcept`s
- `beneficial` is dropped
- `equipment/function` and `ammunition_payload` already have approved split directions
- `creature/threat_profile` already moved away from overusing `*_application`

## Related

- [Derived-tag ontology future shape](./derived-tag-ontology-future-shape.md)
- [Derived Tag Canonical Concept Model](../../../scratch/plans/derived-tag-concept-model.md)
- [Derived Tag Concept Mapping Summary](../../../scratch/plans/derived-tag-concept-mapping-summary.md)
