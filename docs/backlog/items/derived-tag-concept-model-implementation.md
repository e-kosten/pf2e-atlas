# Derived-Tag Concept Model Implementation

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

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
