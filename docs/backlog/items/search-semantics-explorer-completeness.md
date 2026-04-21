# Search Semantics Explorer Completeness

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

A remaining dirty worktree, `feat/search-semantics-completeness-track1`, contains a substantial feature direction for the ontology/search-semantics explorer but an implementation that no longer matches the current architecture.

That scratch work concentrated a large feature expansion in:

- `src/app/ontology-service.ts`

The core product idea is still valuable: the search-semantics explorer should be a richer live exploration surface over the real corpus, not a comparatively thin static browse tree.

However, the current patch is not suitable for direct merge because:

- `main` has since reduced `src/app/ontology-service.ts` to a thin facade and moved actual domain assembly into `src/app/ontology/search-semantics-domain.ts`
- the worktree directly opens SQLite with `DatabaseSync` inside `ontology-service`, which does not match the current documented boundary for search-semantics loading and app-layer ownership

## Desired Outcome

Extend the search-semantics explorer so it better supports real exploration of the indexed corpus from one coherent live surface.

That future implementation should aim to provide:

- richer scoped browsing by category and subcategory
- deeper field/value inspection with better detail lines and context
- live record-backed exploration from concrete leaves, not just abstract taxonomy browsing
- actor/item metric exploration that helps users understand what metrics exist, how they vary, and where they apply
- better continuity between:
  - search-semantics browsing
  - understanding the live corpus
  - launching real result/query flows from explored nodes

## Constraints

- Do not restore the old monolithic `src/app/ontology-service.ts` ownership model. Keep `ontology-service` as the facade and implement real behavior in the current `src/app/ontology/` owners.
- Do not open SQLite directly from `src/app/ontology-service.ts` or equivalent app-layer feature logic unless the architecture is explicitly revised to permit it.
- Keep search-semantics loading on shared facades and documented storage/data boundaries.
- Preserve the accepted direction that search semantics should remain a live exploration surface connected to the real corpus, not drift back toward example-only or sample-only nodes.
- Keep returned ontology models readonly and avoid coupling UI-specific mutable state to shared ontology node objects.

## Notes

### Source context

This item preserves the useful feature intent from the dirty worktree `feat/search-semantics-completeness-track1`.

The worktree appears to have been trying to add all of the following:

- scoped search-semantics nodes keyed by category/subcategory combinations
- richer value inspection and scope-aware detail copy
- live record-node loading for explored metadata/value nodes
- actor metric and item metric grouping, key browsing, and value browsing
- more complete “inspect what this means in the live corpus” flows from ontology/search-semantics nodes

### Why the idea matters

This is aligned with the accepted architectural direction that search semantics, ontology inspection, and query entry should converge on one shared live exploration surface.

The product value is:

- users can understand the real searchable space more directly
- users can move from abstract search semantics to concrete live records more easily
- metric-heavy or field-heavy areas become explorable instead of opaque
- ontology/search-semantics browsing becomes more useful as a learning and query-seeding tool

### Why the current implementation should not be merged as-is

- It targets the older monolithic `src/app/ontology-service.ts` rather than the current split ownership under `src/app/ontology/`.
- It directly opens SQLite via `DatabaseSync`, which conflicts with the current boundaries around search-semantics loading and app/storage ownership.
- It would need to be re-expressed through the current ontology builder/facade structure rather than replayed as a literal patch.

### Implementation intent

A future implementation should start from the current architecture:

1. keep `createPf2eApplicationOntologyService()` as the facade in `src/app/ontology-service.ts`
2. extend the current search-semantics domain builders under `src/app/ontology/`
3. route any needed live corpus or metric access through the appropriate shared data/storage boundary instead of direct feature-local DB opening
4. expose richer explorer nodes, value nodes, and record-backed detail through the existing readonly ontology model surface

The goal is to preserve the feature ambition while fitting the current owner split.

### Comparison to current `main`

Current `main` has the right structural ownership but a thinner explorer implementation.

The deferred work from this item would make the explorer:

- more corpus-aware
- more scope-aware
- more complete around metrics and value inspection
- better at connecting abstract search semantics to concrete record exploration

So the gap is not “missing architecture.” The gap is “missing feature depth inside the current architecture.”

### Suggested validation checks for a future implementation

- verify that concrete search-semantics leaves can launch or inspect real result sets where appropriate
- verify that metric/value exploration works across category/subcategory scopes without breaking readonly ontology model assumptions
- verify that new explorer depth does not regress initial ontology route readiness or turn route entry back into post-mount loading drift
- add tests around the new search-semantics domain builders rather than only around final UI behavior

## Related

- [Actor metrics search orchestration](./actor-metrics-search-orchestration.md)
- [Structured query summary model](./structured-query-summary-model.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [ADR 0005: Live search-semantics exploration](../../architecture/decisions/0005-live-search-semantics-exploration.md)
- [ADR 0007: Render-ready route transitions](../../architecture/decisions/0007-render-ready-route-transitions.md)
- [ADR 0002: Readonly ontology and explicit storage boundary](../../architecture/decisions/0002-readonly-ontology-and-explicit-storage-boundary.md)
