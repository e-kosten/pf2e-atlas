# Filter Shape Convergence

Status: in_progress  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-24

## Problem

Most of the shared search-contract cleanup has already landed, but the remaining public follow-through is narrower than this item originally described.

The canonical `SearchRequest` model now lives on `mode` plus optional `search` plus root `filter`, and the TUI has already moved to that model. The remaining gap is finishing the MCP-facing follow-through everywhere it still presents or documents older surface assumptions, especially the lookup-specific public shape and any stale history/doc wording that still implies flat root filters or `parts`-based canonical state.

## Desired Outcome

Finish the remaining public-surface convergence work so MCP, TUI, and docs all describe the same durable search contract.

The remaining work should:

- keep MCP transport and discovery wording aligned with the canonical `mode` / `search` / `filter` model
- finish lookup-specific MCP follow-through without reopening alias request shapes
- keep backlog/history wording honest about the landed `SearchRequest` and filter-tree ownership model

## Constraints

- Do not add compatibility readers or alias request fields for the replaced flat transport shape.
- Keep the canonical search/filter pipeline shared rather than creating surface-specific parallel models.
- Coordinate with the landed TUI search-state ownership captured in ADR 0013 and the updated history notes for earlier TUI cleanup work.

## Related

- [Search architecture](../../architecture/search.md)
- [TUI architecture](../../architecture/tui.md)
- [Structured query summary model](../history/items/structured-query-summary-model.md)
