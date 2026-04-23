# Search Filter Explorer Draft Canonicalization

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

This item is complete. It remains here as durable context for the ownership decisions that landed during the boundary-restoration cleanup on 2026-04-21.

A now-retired dirty worktree, `fix/tui-cleanup-slice`, contained a useful cleanup direction for TUI search state and filter-explorer draft ownership, but the patch itself is not suitable for direct merge.

That scratch work mixed together several concerns, some of which have already landed on `main`:

- shared policy presentation moved into `src/tui/framework/`
- ontology presenter and explorer-cache ownership moved under `src/app/ontology/`
- old TUI-local ontology wrapper files were removed

The useful part of that worktree is now fully landed:

- `query.filters.parts` should stay the canonical TUI structured-query representation
- TUI search state should not keep parallel compatibility fields for subcategory, level range, rarity, action cost, and metadata when those values can be derived from query parts
- filter-explorer compose state should converge on one durable draft shape instead of carrying side-channel compatibility fields such as `structuredMetadata`
- workflow/session ownership should carry exploration scope such as `scopedFields`, rather than baking that scope into the draft object itself

## Desired Outcome

This outcome is now landed.

The landed cleanup provides:

- `query.filters.parts` as the sole canonical TUI structured-query representation
- one canonical filter-explorer compose draft shape
- metadata owned directly by the compose draft when compose mode needs it
- workflow/session-level ownership of scope such as `scopedFields`
- no extra compatibility-only draft fields that exist only to bridge older search-screen shapes

## Constraints

- Do not replay the dropped worktree literally. Several edited paths there no longer match the current split `src/tui/search-screen/` layout.
- Preserve the current architecture where shared policy presentation lives in `src/tui/framework/` and ontology presenter/cache ownership lives under `src/app/ontology/`.
- Keep `query.filters.parts` canonical rather than reintroducing parallel field copies for convenience.
- Do not hide this cleanup inside a renderer-only feature. If summary/document rendering work needs this foundation, make the state cleanup explicit.
- Keep filter-explorer mode boundaries clear: compose state should own compose concerns, while workflow/session objects own launch scope and host-specific context.

## Notes

### Source context

This item preserves the unique useful direction from the retired dirty worktree `fix/tui-cleanup-slice`.

The scratch patch suggested all of the following:

- removing duplicated TUI query fields such as `subcategory`, `levelMin`, `levelMax`, `rarity`, `actionCost`, and `metadata` from the canonical search-query shape in favor of deriving them from query parts
- letting `FilterExplorerComposeDraft` carry `metadata` directly instead of relying on a parallel `structuredMetadata` field
- passing `scopedFields` through workflow/session ownership instead of embedding it in the draft shape
- trimming search/filter-explorer seams that still reflected an older compatibility-oriented model

### Why the cleanup mattered

This cleanup mattered because it reduced the number of places where the TUI had to keep equivalent state in sync:

- query-state interpretation becomes simpler
- filter-explorer compose mode becomes easier to reason about
- future search-screen rendering work can depend on one model instead of bridging multiple transitional shapes
- type signatures become clearer because they describe one owner shape instead of a compatibility bundle

This is especially relevant to future work on richer structured-query rendering and document-style search editing.

### Validation that landed with the cleanup

- canonical query-state behavior is covered by focused TUI search tests
- filter-explorer draft/query round-tripping is covered by search-service tests
- the shared TUI search/filter-explorer seams now reflect current durable ownership instead of transitional state bridging

### Relationship to other backlog items

This is related to, but distinct from, [Structured query summary model](./structured-query-summary-model.md).

- this item is about canonical state ownership and draft shape
- the structured-query summary item is about introducing a richer intermediate summary/document model for rendering and stable section identity

In practice, the landed cleanup makes that later summary-model work easier and less error-prone.

## Related

- [Structured query summary model](./structured-query-summary-model.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [TUI architecture](../../../architecture/tui.md)
- [Architectural boundaries](../../../architecture/boundaries.md)
- [Search architecture](../../../architecture/search.md)
