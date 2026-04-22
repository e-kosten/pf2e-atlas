# Remove isUnique Metadata

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

`isUnique` is currently exposed as metadata even though it duplicates meaning already carried by rarity.

That creates unnecessary surface area in the searchable metadata model and makes the explorer/filter vocabulary look more complex than it needs to be.

## Desired Outcome

Remove `isUnique` as a standalone metadata concept wherever it is only duplicating rarity semantics.

## Constraints

- Do not remove any distinct behavior if a current caller truly depends on something beyond rarity.
- Keep MCP and TUI semantics aligned; this should not disappear from one surface and remain as a first-class filter on the other.
- Prefer direct replacement and cleanup over leaving behind compatibility vocabulary unless a real external dependency requires it.

## Related

- [Search architecture](../../architecture/search.md)
- [Filter shape convergence](./filter-shape-convergence.md)
