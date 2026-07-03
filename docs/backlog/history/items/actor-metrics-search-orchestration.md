# Actor Metrics Search Orchestration

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

This item is complete. It remains here as durable context for the metric-query path that is now implemented across the shared search runtime, MCP surface, and TUI compose flows.

## Desired Outcome

That outcome is now landed:

- shared search filters support actor/item metric predicates and compare predicates
- MCP search semantics exposes those predicates and discovery paths
- TUI inspect/compose flows can turn metric selections into executable search metadata
- integration and UI tests cover the end-to-end behavior

## Constraints

- Keep any future metric-search follow-up on the shared search/filter pipeline.
- Do not reintroduce a gap between MCP and TUI metric-query semantics.

## Related

- [Search architecture](../../../architecture/node/search.md)
- [TUI architecture](../../../architecture/node/tui.md)
