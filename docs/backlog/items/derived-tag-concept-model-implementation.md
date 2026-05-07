# Derived-Tag Concept Model Implementation

Status: in_progress  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-07

## Problem

The future ontology shape is now partially implemented in tracked source, but the implementation is still not at the final authored end state.

The runtime, editorial tooling, and TUI now consume live concept/projection/translation plumbing, but canonical concepts and projections are still compiled from the current category-local ontology rather than authored independently as the final source of truth.

## Desired Outcome

Move the derived-tag concept/projection model into tracked source ownership so the future-state ontology is implemented rather than only preserved in planning artifacts.

That implementation work should cover:

- a durable source-of-truth schema for canonical concepts and category projections
- tracked `src/tags/` ownership for the concept/projection model
- explicit aggregate concept and relation authoring
- canonical metadata that editorial tooling and the TUI can surface directly
- a clear replacement path from the current category/axis/family/tag shape to the canonical concept/projection model

## Current Slice

The first live cutover slice is landed in tracked `src/tags/` source.

Implemented in this slice:

- canonical concept, projection, relation, and translation-record types
- a live `src/tags/translations/` layer that bridges current ontology tags into canonical concepts and projections
- projection-backed ontology publication wired into `runtime/derivation/api.ts`
- stable user/runtime tag ids preserved while canonical labels and metadata publish alongside them
- editorial/runtime accessors for concept-model and translation-record inspection
- a dedicated ontology translation review route in the TUI
- session-backed ontology translation review artifacts under `scratch/translation-sessions/`
- explicit lint-and-import writeback for ontology translation review
- translation import support for both per-tag overrides and promoted family-default updates

Intentionally deferred to a follow-up slice:

- direct assignment migration onto canonical concept/projection ownership
- retirement of legacy rules and legacy seed migration paths
- independent authored source ownership for canonical concepts and projections instead of compiler-derived publication from the current category ontology

This means the future ontology model is no longer planning-only, and the human translation-review workflow is live, but the authored source model is still transitional because the canonical layer is not yet independently authored.

## Constraints

- Keep one canonical semantic label per concept. Do not allow projection-local semantic renames.
- Keep legacy naming compatibility in `src/tags/legacy-*` only, not in the canonical model.
- Keep browse scaffolding and semantic ownership separate: projections own browse placement, not semantic identity.
- Do not land a parallel long-term model beside the current one without a clear replacement plan.
- Coordinate with [Derived-tag assignments layout](./derived-tag-assignments-layout.md); assignment layout and concept-model ownership should not drift apart.

## Notes

The documentation-preservation item [Derived-tag ontology future shape](./derived-tag-ontology-future-shape.md) is complete and should remain as the durable design record for the model and the resolved review outcomes.

This item now tracks the remaining implementation gap between the landed live cutover and the final independent authored end state.

## Related

- [Derived-tag assignments layout](./derived-tag-assignments-layout.md)
- [Derived-tag ontology future shape](./derived-tag-ontology-future-shape.md)
- [Derived-tag ontology next review questions](./derived-tag-ontology-next-review-questions.md)
- [Derived tag manifest tooling metadata](./derived-tag-manifest-tooling-metadata.md)
- [Editorial architecture](../../architecture/editorial.md)
