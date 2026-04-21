# Filter Explorer Internal Decomposition

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

The boundary-restoration cleanup fixed the most important ownership bug in the search/filter-explorer path: canonical query and compose-draft ownership now live in the right places. What it did not do was finish the internal decomposition of the shared filter explorer itself.

Two modules are still carrying too many concerns at once:

- `src/tui/filter-explorer/controller.ts`
- `src/tui/filter-explorer/search-draft.ts`

Today `controller.ts` still mixes several owners in one file:

- reducer and browser-state transitions
- inspect-result derivation
- scalar-predicate compilation and summary formatting
- compose-mode state wiring
- screen controller/runtime orchestration

Likewise, `search-draft.ts` still combines several different responsibilities:

- metadata policy translation
- ontology-backed field and target resolution
- draft preparation from query or metadata trees
- metadata serialization back out of draft state
- query reapplication logic

That is not a correctness bug in the same way the old canonical-state problem was, but it still leaves the shared filter explorer as a subsystem sink. If future work keeps landing in these files, the current architecture will drift back toward mixed ownership even though the higher-level state model is now correct.

## Desired Outcome

Split the shared filter explorer into clearer internal owners without reintroducing compatibility layers or changing the current public behavior.

A future implementation should aim for:

- controller/runtime wiring separated from pure state transitions
- inspect-mode result compilation separated from general navigation state
- scalar-clause compilation and formatting separated from the main controller body
- draft preparation and metadata serialization separated from ontology-model shaping
- target-resolution logic separated from draft persistence logic

The goal is not file count for its own sake. The goal is to make it obvious where future changes belong so the shared explorer can evolve without one file becoming the owner of everything.

## Constraints

- Preserve the current shared filter-explorer role. This is still the durable browse/compose explorer used by multiple hosts, not a search-screen-only implementation.
- Do not reopen the canonical draft/query ownership question that was already closed by [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md).
- Do not introduce compatibility wrappers or temporary forwarding modules just to land the split.
- Keep the shared TUI interaction and list/detail presentation architecture intact while decomposing internals.
- Prefer moving existing logic behind explicit internal owners over inventing a new abstraction layer with no concrete caller benefit.

## Notes

### Current hotspot shape

The current code points to two specific hotspots:

- `src/tui/filter-explorer/controller.ts`
- `src/tui/filter-explorer/search-draft.ts`

Useful seams already exist conceptually in the current implementation even though they are not fully split yet:

- reducer/state-transition logic
- inspect-result and scalar-query compilation
- compose-draft preparation and reapplication
- ontology target resolution
- metadata translation helpers

That means this item is not a speculative redesign. It is mostly a matter of turning already-distinct responsibilities into explicit module owners.

### Why the cleanup matters

This work would improve maintainability more than visible product behavior.

It matters because the filter explorer now sits at the center of several workflows:

- search-screen structured editing
- ontology-backed inspect/browse flows
- shared list/detail presentation

If those flows continue to add behavior into the current large files, future work will have to rediscover implicit boundaries each time. Splitting the owners makes the architecture easier to preserve and easier to enforce with focused tests or lint rules later.

### Suggested implementation shape

One reasonable direction would be to pull apart the current owners into smaller focused modules such as:

- controller state/reducer helpers
- inspect compilation helpers
- scalar-clause formatting/translation helpers
- draft-preparation helpers
- metadata serialization/reapplication helpers
- ontology target-resolution helpers

The exact filenames are less important than keeping each owner focused and making the resulting imports read like architecture rather than like shared utility sprawl.

### Candidate starting points

- `src/tui/filter-explorer/controller.ts`
- `src/tui/filter-explorer/search-draft.ts`
- `src/tui/filter-explorer/workflow-actions.ts`
- `src/tui/filter-explorer/types.ts`
- `tests/tui/filter-explorer-controller.test.tsx`
- `tests/tui/search-screen.test.tsx`

### Suggested validation checks for a future implementation

- verify that compose and inspect flows still behave the same after the split
- add or tighten tests around pure reducer/state-transition helpers if those are extracted
- add focused tests around scalar-clause compilation and query-open behavior if those helpers are extracted
- verify that draft preparation, metadata serialization, and query reapplication still round-trip correctly
- confirm that no new compatibility shims or duplicate helper pathways were introduced during the decomposition

## Related

- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
- [Structured query summary model](./structured-query-summary-model.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [ADR 0009: Shared list/detail presentation layer](../../architecture/decisions/0009-shared-list-detail-presentation-layer.md)
