# Casting Time Filter Gap

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

The current search surface exposes numeric `actionCost`, but it does not model non-action casting times such as `1 minute` or `10 minutes` as a first-class searchable concept.

That leaves a gap for spells and similar records whose activation/casting timing is meaningful but not representable through the current promoted filter model.

## Desired Outcome

Add a searchable casting-time concept that covers longer or non-standard casting times without overloading `actionCost`.

## Constraints

- Do not reopen the current filter-shape convergence pass; this is a follow-up gap, not part of the current implementation block.
- Keep the eventual shape aligned with the normalized shared search model rather than adding a one-off MCP-only field.
- Decide explicitly whether casting time belongs as promoted filter language or metadata-owned predicate vocabulary.

## Related

- [Filter shape convergence](../history/items/filter-shape-convergence.md)
- [Remove isUnique metadata](../history/items/remove-isunique-metadata.md)
