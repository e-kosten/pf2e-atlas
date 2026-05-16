# Structured Query Summary Model

Status: done  
Priority: n/a  
Owner: unassigned  
Last reviewed: 2026-04-24

## Outcome

The live search workspace now derives a first-class summary/document model from canonical query state before rendering editor rows or query summaries.

The landed shape is:

- the workspace summary/document model derives from the durable canonical query state owned by `SearchRequest`
- today that means `SearchRequest` plus `query.filter`, not the older `query.filters.parts` state model that existed when this item first landed
- `src/tui/search-screen/workspace/query-summary.ts` derives a stable summary/document model from that canonical state
- `src/tui/search-screen/workspace/workspace.ts` renders workspace rows and summary/detail panes from that summary model instead of re-deriving structured meaning from raw query state in each helper

The summary/document layer provides:

- a first-class summary representation for the current structured query
- stable row/section identity for major structured-query concepts such as profile, category, level range, rarity, action cost, and metadata clauses
- explicit anchors for metadata nodes and nested query structure
- a clean seam between:
  - query-state interpretation
  - summary/document modeling
  - visible editor row rendering

This closes the remaining architectural gap between canonical query ownership and editor-facing rendering. ADR 0013 later tightened the canonical-owner side of that same seam by making `SearchRequest` the long-lived TUI search state.

## Notes

### Why this matters

The landed summary/document layer makes the search query easier to treat as a document rather than only as a live list of editor rows. That supports:

- richer structured-query detail panes
- stable targeting of logical sections during keyboard navigation
- clearer inactive placeholders and optional sections
- alternate renderers for the same query state
- future document-style query rendering without coupling presentation directly to low-level query-state extraction

### Landed validation shape

The implementation includes focused validation around:

- stable structured-query identities when visible row order changes
- stable metadata anchors by logical path rather than visible position
- workspace rendering consuming the same summary model used by structured-query summary output

## Related

- [Backlog: Search interaction cleanup](../../backlog.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
- [View pages and detail presentation](../../items/view-pages-and-details.md)
- [TUI architecture](../../../architecture/node/tui.md)
- [Search architecture](../../../architecture/node/search.md)
- [Architectural boundaries](../../../architecture/node/boundaries.md)
- [ADR 0006: Shared TUI interaction contracts](../../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
- [ADR 0009: Shared list/detail presentation layer](../../../architecture/decisions/0009-shared-list-detail-presentation-layer.md)
