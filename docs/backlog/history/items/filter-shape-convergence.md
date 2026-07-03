# Filter Shape Convergence

Status: done  
Priority: n/a  
Owner: unassigned  
Last reviewed: 2026-04-24

## Outcome

The shared search contract now converges on one canonical `SearchRequest` model across MCP, TUI, and supporting documentation.

The landed state is:

- the public request model is expressed on `mode`, optional `search`, and root `filter`
- the canonical filter model is one boolean tree using `anyOf`, `allOf`, and `not`
- browse, search, and lookup use explicit mode-specific request and sort semantics instead of a loose flat transport shape
- the TUI keeps canonical `SearchRequest` state directly rather than a parallel `parts`-based semantic model
- lookup presentation uses explicit `matchType` metadata through the shared result-view pathway instead of a lookup-only list/detail implementation

## Notes

This cleanup landed as direct replacement work rather than a compatibility migration:

- no alias request fields were kept for retired flat-root transport shapes
- no shared/public `parts` query model survives beside canonical `SearchRequest`
- lookup-specific grouping and badge behavior is expressed through the shared result-view seam, not a lookup-only presentation stack

## Related

- [Search architecture](../../../architecture/node/search.md)
- [TUI architecture](../../../architecture/node/tui.md)
- [Structured query summary model](./structured-query-summary-model.md)
- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
