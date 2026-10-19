# Derived-Tag Ontology Future Shape

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

Preservation update, 2026-10-19: this is historical Node design context. Its
“approved” and “resolved” language describes that experiment, not current Rust
requirements. The [derived-tag research package](../../../research/derived-tags/README.md)
now preserves the local-only artifacts and later discussion. Feature development
is [deferred](../../items/rust-derived-tag-redesign.md) pending demonstrated gaps
in current native filters and semantic search.

## Problem

This item is complete as a documentation-preservation task.

It now provides one durable tracked place that explains:

- the agreed long-term canonical model
- the major decisions already locked in
- the remaining ontology areas that still need focused review
- how this work relates to later implementation of assignments, schema changes, and TUI/editorial behavior

That preservation work matters because the scratch artifacts are intentionally temporary working plans rather than the main long-term reminder surface.

## Desired Outcome

That outcome is now landed. Later sessions can resume from a durable tracked record instead of reconstructing the ontology work from chat history alone.

This item now captures:

- the current preferred canonical concept model
- the key resolved naming and modeling decisions already reflected in the scratch mapping
- the remaining medium-confidence ontology families that still deserve review
- the likely future implementation path from scratch planning artifacts into real source ownership

## Constraints

- Keep the user-facing ontology based on one shared canonical label per concept. Do not allow category projections to drift into local semantic renames.
- Keep legacy naming compatibility in `src/tags/legacy-*` only, not in the canonical ontology model.
- Keep browse scaffolding in `browseAxis` and `family`; exposed composite tags should only exist when they are meaningful filter targets.
- Treat exposed composite tags as canonical `AggregateConcept`s, not as projection-only pseudo-tags.
- Keep the bounded operation vocabulary for operational concepts and derive canonical ids from semantic `(domain, operation)` choices rather than ad hoc labels.
- Do not reopen resolved areas such as `regional_setting`, `creature_family`, `outbreak_containment`, and aggregate-composite handling unless new evidence justifies it.
- Do not expand this backlog item into full implementation work. The scratch plans remain the active working surface for row-by-row ontology mapping.

## Notes

### Current Canonical Model

The current preferred semantic shape is:

- `OperationalConcept`
- `DescriptiveConcept`
- `AggregateConcept`
- `DomainConcept`
- `ConceptRelation`
- `CategoryProjection`

Key semantic principles:

- `CategoryProjection` owns browse placement, description, and assignment guidance, but not semantic naming
- `DescriptiveConcept` uses structured facets such as `setting`, `theme`, `creature_family`, `role`, `delivery`, `progression`, `behavior_override`, `response_demand`, `challenge_structure`, `capability`, `effect`, `function`, `mechanism`, and `pathogenesis`
- `OperationalConcept` uses a bounded operation vocabulary such as `discover`, `remediate`, `remove`, `contain`, `clean_up`, `counteract`, `expel`, `appease`, `sanctify`, `resolve`, `disarm`, and `bypass`
- `AggregateConcept` is for meaningful filterable umbrella concepts such as `communication`, `infiltration`, `reconnaissance`, `problem_discovery`, and `problem_resolution`

### Preserved Working Artifacts

The former local working artifacts are now preserved as inert snapshots:

- [Derived Tag Canonical Concept Model](../../../research/derived-tags/sources/node-concept-model/derived-tag-concept-model.md)
- [Derived Tag Concept Mapping CSV](../../../research/derived-tags/sources/node-concept-model/derived-tag-concept-mapping.csv.md)
- [Derived Tag Concept Mapping Summary](../../../research/derived-tags/sources/node-concept-model/derived-tag-concept-mapping-summary.md)
- [Generator provenance and retention limits](../../../research/derived-tags/sources-and-cleanup.md)

Those snapshots preserve the historical proposals; they are not live plans or
an approved taxonomy for a revived implementation.

### Major Resolved Decisions

The following areas have already been resolved in the scratch planning and mapping work:

- `regional_setting` stays as a setting-first concept space and is considered resolved as-is
- `bound_object` and `ontology_cluster` now map into a shared `creature_family` semantic space
- `outbreak_response` was dropped in favor of `outbreak_containment`
- `epidemiological_profile` is a heterogeneous browse family under `disease_model`, not a single semantic kind
- exposed composite tags are canonical `AggregateConcept`s rather than projection-only pseudo-tags
- `beneficial` should be dropped rather than normalized
- equipment `ammunition_payload` should split across existing facet kinds:
  - `elemental_payload`, `explosive_payload`, `spell_payload` -> `delivery`
  - `creature_bane` -> `capability`
- `equipment/function` should split across remediation/support/capability rather than remain one semantic bucket
- `creature/threat_profile` should not overuse `*_application`; several rows now normalize into more specific operational shapes such as `ambush_grab`, `prey_control`, `regeneration`, `reinforcement`, `terrain_control`, and `infiltration`

### Current Mapping State

As of the last review on 2026-04-21, the scratch mapping summary reports:

- `aggregate`: 26
- `descriptive`: 323
- `drop`: 1
- `operational`: 165

The remaining mapping medium-confidence count at that point is `200`.

That number is no longer concentrated in creature or affliction work. The remaining medium rows are in spell, hazard, and equipment families that are mostly broad family-level mappings rather than obviously broken row-level decisions.

### Remaining Focus Areas

The main remaining review targets are:

- `spell/support`
  - broad mix of remediation, protection, survivability, and enhancement concepts
- `spell/control`
  - broad operational family spanning denial, battlefield shaping, countermagic, and pressure concepts
- `hazard/countermeasure_profile`
  - high-value cross-category answer-path mapping family
- `hazard/environmental_danger`
  - broad hazard-side effect/application family where some rows may still want sharper normalization
- `equipment/anti_magic`
  - small but important family because `countermagic` and `magic_protection` are near each other in browse space but may not be the same semantic kind

Additional remaining medium families are mostly broad utility/effect clusters in spell, equipment, and hazard spaces. Many of those may resolve through one or two family-level validation passes rather than through major model changes.

The retired next-review-questions item is recoverable through the
[source map](../../../research/derived-tags/sources-and-cleanup.md). Its families
remain historical questions, not a current work queue.

### Likely Future Implementation Work

The Node implementation follow-through is historical. Current capability intent
is tracked in the [deferred Rust item](../../items/rust-derived-tag-redesign.md).

When this backlog item is picked up for implementation rather than continued ontology review, the expected work will likely include:

1. deciding the real source-of-truth schema for canonical concepts and category projections
2. moving the scratch concept/projection model into tracked source ownership under `src/tags/`
3. teaching assignment tooling and the TUI how to surface canonical metadata such as concept kind, operation, domain, and relations
4. deciding how aggregate concepts are authored and how their member relationships are declared
5. finishing row-by-row ontology cleanup in the remaining medium-confidence families before large-scale assignment work resumes

## Related

- [Derived-tag research and history](../../../research/derived-tags/README.md)
- [Historical sources and cleanup boundaries](../../../research/derived-tags/sources-and-cleanup.md)
- [Deferred Rust derived-tag workstream](../../items/rust-derived-tag-redesign.md)
- [Derived Tag Canonical Concept Model](../../../research/derived-tags/sources/node-concept-model/derived-tag-concept-model.md)
- [Derived Tag Concept Mapping Summary](../../../research/derived-tags/sources/node-concept-model/derived-tag-concept-mapping-summary.md)
