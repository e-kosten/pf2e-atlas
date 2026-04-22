# Search Semantics Explorer Completeness

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

Most of the broader search-semantics explorer depth that this item originally preserved is now landed on `main`.

Scoped browsing, live field/value exploration, record-backed leaves, and actor/item metric discovery are all already present in the current ontology builder architecture.

The remaining open gap is narrower:

- numeric metric keys still fall back to inspect/query-style exploration instead of offering richer live corpus value exploration
- the live value-listing path still supports text and boolean metric values but not numeric metric value exploration
- the backlog item still describes this as a broad explorer-completeness problem instead of the specific remaining follow-through

## Desired Outcome

Finish the remaining numeric-metric exploration follow-through inside the existing ontology/search architecture.

That future implementation should aim to provide:

- a richer live-corpus exploration path for numeric metric keys
- meaningful inspection or bucket/range-oriented exploration for numeric metric spaces
- continuity between numeric metric discovery, query seeding, and result launching
- no regression of the already-landed scoped browsing, live record leaves, and metric discovery behaviors

## Constraints

- Keep `ontology-service` as the facade and implement behavior in the current `src/app/ontology/` owners.
- Do not open SQLite directly from `src/app/ontology-service.ts` or equivalent feature-local app code unless the architecture is explicitly revised.
- Keep search-semantics loading on shared facades and documented storage/data boundaries.
- Preserve the already-landed readonly ontology model and live record-backed explorer behaviors.
- Do not reopen already-landed explorer depth as if it were still missing.

## Notes

### Current Status

The older `feat/search-semantics-completeness-track1` worktree preserved a useful product direction, but most of that direction has since landed through the current split ownership model.

Current `main` already provides:

- scoped search-semantics nodes keyed by category/subcategory combinations
- live record-backed exploration from metadata/value nodes
- actor metric and item metric grouping, namespace browsing, key browsing, and query launch
- shared inspect/open behavior that routes explored leaves into real search/result flows

### Remaining Gap

The main remaining gap is numeric metric value exploration. Numeric metric keys can already seed queries and route into the shared scalar editor path, but they do not yet expose richer live corpus value-space exploration comparable to the current text/boolean metric value-listing behavior.

### Implementation intent

A future implementation should:

1. extend the current search-semantics helpers and domain builders under `src/app/ontology/`
2. reuse the existing shared data/storage boundaries rather than introducing feature-local DB access
3. decide how numeric metric spaces should be explored meaningfully without pretending they are enumerable text-value lists
4. preserve the current inspect/open and route-preparation behavior while adding richer numeric metric exploration

### Suggested validation checks for a future implementation

- verify that numeric metric exploration works across category/subcategory scopes without breaking readonly ontology model assumptions
- verify that concrete metric leaves still launch or inspect real result sets where appropriate
- verify that new metric exploration does not regress initial ontology route readiness or turn route entry back into post-mount loading drift
- add tests around the new search-semantics domain builders and value-exploration behavior rather than only final UI behavior

## Related

- [Structured query summary model](./structured-query-summary-model.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [ADR 0005: Live search-semantics exploration](../../architecture/decisions/0005-live-search-semantics-exploration.md)
- [ADR 0007: Render-ready route transitions](../../architecture/decisions/0007-render-ready-route-transitions.md)
- [ADR 0002: Readonly ontology and explicit storage boundary](../../architecture/decisions/0002-readonly-ontology-and-explicit-storage-boundary.md)
