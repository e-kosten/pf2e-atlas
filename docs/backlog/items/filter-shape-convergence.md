# Filter Shape Convergence

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

The filter model has drifted between the MCP and TUI surfaces.

The original scratch note called out one concrete symptom: the MCP side still treats rarity and level as more special than other metadata filters, while the TUI is moving toward a more unified structured-query model.

That drift makes it harder to keep search semantics, editor behavior, and surface documentation aligned.

## Desired Outcome

Bring the MCP and TUI filter models back into convergence so the same concepts are represented coherently across both surfaces.

That work should:

- reduce unnecessary “special case” treatment for filters like rarity and level
- keep shared search semantics aligned with the live editor/query model
- make it easier for query seeding, exploration, and transport layers to describe the same structure

## Constraints

- Avoid breaking stable external surface behavior without a clear migration story.
- Keep the canonical search/filter pipeline shared rather than creating surface-specific parallel models.
- Coordinate with the landed TUI query-model work captured in [Structured query summary model](../history/items/structured-query-summary-model.md).

## Related

- [Search architecture](../../architecture/search.md)
- [TUI architecture](../../architecture/tui.md)
- [Structured query summary model](../history/items/structured-query-summary-model.md)
