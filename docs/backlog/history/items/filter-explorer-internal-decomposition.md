# Filter Explorer Internal Decomposition

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

This item is complete. It remains here as durable context for the owner split that landed during the filter-explorer boundary-closure refactor on 2026-04-21.

Before that refactor, two modules were still carrying too many concerns at once:

- `src/tui/filter-explorer/controller.ts`
- `src/tui/filter-explorer/search-draft.ts`

At that point, `controller.ts` still mixed several owners in one file:

- reducer and browser-state transitions
- inspect-result derivation
- scalar-predicate compilation and summary formatting
- compose-mode state wiring
- screen controller/runtime orchestration

Likewise, `search-draft.ts` still combined several different responsibilities:

- metadata policy translation
- ontology-backed field and target resolution
- draft preparation from query or metadata trees
- metadata serialization back out of draft state
- query reapplication logic

That was not a correctness bug in the same way the old canonical-state problem was, but it still left the shared filter explorer as a subsystem sink.

## Desired Outcome

This outcome is now landed.

The landed split now provides:

- controller/runtime wiring separated from pure state transitions
- inspect-mode result compilation separated from general navigation state
- scalar-clause compilation and formatting separated from the main controller body
- draft preparation and metadata serialization separated from ontology-model shaping
- target-resolution logic separated from draft persistence logic

The goal was not file count for its own sake. The landed change makes it clearer where future filter-explorer work belongs so one file does not silently become the owner of everything again.

## Constraints

- Preserve the current shared filter-explorer role. This is still the durable browse/compose explorer used by multiple hosts, not a search-screen-only implementation.
- Do not reopen the canonical draft/query ownership question that was already closed by [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md).
- Do not introduce compatibility wrappers or temporary forwarding modules just to land the split.
- Keep the shared TUI interaction and list/detail presentation architecture intact while decomposing internals.
- Prefer moving existing logic behind explicit internal owners over inventing a new abstraction layer with no concrete caller benefit.

## Notes

### Landed owner split

The mixed owners were split into durable internal modules such as:

- controller state and reducer ownership
- inspect/open compilation ownership
- route-handling ownership
- draft/query preparation and serialization ownership
- ontology model/target-resolution ownership
- field-policy translation ownership

### Why the cleanup matters

This work would improve maintainability more than visible product behavior.

It matters because the filter explorer now sits at the center of several workflows:

- search-screen structured editing
- ontology-backed inspect/browse flows
- shared list/detail presentation

The landed refactor matters because filter-explorer behavior can now evolve without reopening the same mixed-owner hotspot.

### Validation that landed with the refactor

- focused controller-state tests
- focused inspect/open tests
- search-service and draft/query round-trip coverage
- full `npm run build`
- full `cd scripts && npm test`

## Related

- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
- [Structured query summary model](./structured-query-summary-model.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [TUI architecture](../../../architecture/tui.md)
- [Architectural boundaries](../../../architecture/boundaries.md)
- [ADR 0009: Shared list/detail presentation layer](../../../architecture/decisions/0009-shared-list-detail-presentation-layer.md)
