# Remove isUnique Metadata

Status: done  
Priority: n/a  
Owner: unassigned  
Last reviewed: 2026-04-24

## Outcome

`isUnique` is no longer part of the searchable or filterable public metadata vocabulary.

The landed state keeps rarity as the public search/filter concept for uniqueness semantics:

- canonical search/filter inputs do not expose an `isUnique` predicate or first-class filter
- search semantics and discovery surfaces no longer describe `isUnique` as filter vocabulary
- MCP and TUI search flows converge on the same rarity-driven public meaning instead of keeping a duplicated uniqueness facet

Remaining `isUnique` booleans in the repository are record-model or presentation details, not public search/filter vocabulary.

## Notes

This cleanup landed as direct replacement work, not as a compatibility migration:

- no alias metadata field was kept for search callers
- no compatibility reader was added for retired search/filter shapes
- the public contract continues to expose rarity, including the `unique` rarity value, as the durable concept

## Related

- [Filter shape convergence](./filter-shape-convergence.md)
- [Search architecture](../../../architecture/node/search.md)
- [TUI architecture](../../../architecture/node/tui.md)
