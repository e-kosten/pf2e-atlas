# Pointer Target Hit Testing

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

The planned cursor-vs-viewport interaction realignment can add hovered-pane wheel scrolling and click-to-focus without immediately solving fine-grained pointer targeting inside panes.

That still leaves an important missing interaction capability:

- clicking a visible list row to select it directly
- clicking a page section to activate it directly
- clicking an inline or section-local target to enter or activate it directly

Without shared pointer target hit testing, mouse support remains limited to pane-level routing and viewport scrolling. That is still useful, but it falls short of a complete pointer interaction model for result readers and entity pages.

## Desired Outcome

Add shared pointer target hit testing for TUI surfaces that expose stable selectable or activatable regions.

That follow-up should cover:

- shared hit-target modeling for list rows, page sections, and section-local targets where feasible
- click behavior that can both focus the pane and select or activate the clicked target
- rendered-row measurement for wrapped inline spans so pointer targeting and future exact-focus behavior can resolve the concrete terminal row occupied by a selectable span
- shared routing rules so pointer targeting does not become a feature-local screen hack
- compatibility with the broader shared cursor-vs-viewport interaction model

## Constraints

- Do not block initial hovered-pane scrolling or click-to-focus support on fully solving fine-grained hit targeting.
- Do not hardcode pointer targeting separately in search, ontology, and entity-page screens.
- Keep pointer targeting aligned with shared interaction semantics such as `select`, `open`, `preview`, and `back` / `return`.
- Prefer reusable hit-target regions derived from shared presentation models over raw coordinate branching in feature controllers.

## Notes

This item is intentionally separate from the first entity-page implementation pass.

The initial shared mouse/trackpad work should be able to land with:

- hovered-pane wheel or trackpad scrolling
- click-to-focus on panes

This backlog item tracks the richer follow-through where pointer input can also resolve and activate concrete targets inside those panes.

## Related

- [View pages and detail presentation](./view-pages-and-details.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
- [TUI architecture](../../architecture/tui.md)
