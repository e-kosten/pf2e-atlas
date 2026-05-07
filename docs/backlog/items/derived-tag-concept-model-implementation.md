# Derived-Tag Concept Model Implementation

Status: in_progress  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-05-07

## Problem

The durable documentation-preservation work for the derived-tag future shape is complete, but the actual implementation follow-through is still missing.

The canonical concept/projection model and the resolved ontology review decisions now live in backlog and scratch planning context, but tracked source still uses the current category/axis/family/tag ownership shape. The repo does not yet have a real `src/tags/` source-of-truth schema for:

- canonical concepts
- category projections
- aggregate/member relations
- canonical operation/domain metadata

That leaves the future-state ontology model preserved but not yet owned by the runtime, editorial tooling, or TUI surfaces.

## Desired Outcome

Move the derived-tag concept/projection model into tracked source ownership so the future-state ontology is implemented rather than only preserved in planning artifacts.

That implementation work should cover:

- a durable source-of-truth schema for canonical concepts and category projections
- tracked `src/tags/` ownership for the concept/projection model
- explicit aggregate concept and relation authoring
- canonical metadata that editorial tooling and the TUI can surface directly
- a clear replacement path from the current category/axis/family/tag shape to the canonical concept/projection model

## Current Slice

The first live cutover slice is now underway in tracked `src/tags/` source.

Implemented in this slice:

- canonical concept, projection, relation, and translation-record types
- a live `src/tags/translations/` layer that bridges current ontology tags into canonical concepts and projections
- projection-backed ontology publication wired into `runtime/derivation/api.ts`
- stable user/runtime tag ids preserved while canonical labels and metadata publish alongside them
- editorial/runtime accessors for concept-model and translation-record inspection

Intentionally deferred to a follow-up slice:

- a dedicated translation-review session/workbench mode in the TUI
- direct assignment migration onto canonical concept/projection ownership
- retirement of legacy rules and legacy seed migration paths

This means the future ontology model is no longer planning-only, but the human review workflow for unresolved translation rows is still only partially surfaced through summary accessors rather than a full interactive queue.

## Constraints

- Keep one canonical semantic label per concept. Do not allow projection-local semantic renames.
- Keep legacy naming compatibility in `src/tags/legacy-*` only, not in the canonical model.
- Keep browse scaffolding and semantic ownership separate: projections own browse placement, not semantic identity.
- Do not land a parallel long-term model beside the current one without a clear replacement plan.
- Coordinate with [Derived-tag assignments layout](./derived-tag-assignments-layout.md); assignment layout and concept-model ownership should not drift apart.

## Notes

The documentation-preservation item [Derived-tag ontology future shape](./derived-tag-ontology-future-shape.md) is complete and should remain as the durable design record for the model and the resolved review outcomes.

This item exists because the actual source implementation still has to happen.

## Related

- [Derived-tag assignments layout](./derived-tag-assignments-layout.md)
- [Derived-tag ontology future shape](./derived-tag-ontology-future-shape.md)
- [Derived-tag ontology next review questions](./derived-tag-ontology-next-review-questions.md)
- [Derived tag manifest tooling metadata](./derived-tag-manifest-tooling-metadata.md)
- [Editorial architecture](../../architecture/editorial.md)
