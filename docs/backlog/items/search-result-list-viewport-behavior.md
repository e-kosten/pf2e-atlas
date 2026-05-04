# Search Result List Viewport Behavior

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

The current search result reader keeps the left result list cursor-driven. That matches the written entity-page navigation plan, but it does not match the product intent that the active pane might support independent viewport scrolling.

As a result:

- `Ctrl-E` / `Ctrl-Y` only smooth-scroll page/document reading surfaces on the right
- wheel input over the left result pane moves selection rather than an independent list viewport
- the result reader cannot yet support a true "scroll the active pane" model across both panes

This is not necessarily a bug in the current implementation. It is a product and interaction-model follow-up that should be revisited explicitly instead of drifting through ad hoc fixes.

## Desired Outcome

Decide and implement the intended durable behavior for the left result-list pane in the shared search result reader.

The follow-up should answer:

1. Should the left result list remain primarily cursor-driven?
2. Should the active result pane instead support an independently scrollable viewport?
3. If both behaviors are needed, what is the shared interaction contract for selection versus viewport movement on list-like panes?

The end state should be explicit in code, tests, and TUI architecture docs.

## Scope

This item should cover:

- keyboard behavior for `Ctrl-E` / `Ctrl-Y`, page movement, and edge movement in the left result pane
- wheel and trackpad behavior for the left result pane
- help/footer copy so the rendered bindings match the real behavior
- any shared interaction-family consequences for list-like panes beyond the search result reader

This item should not silently broaden into pointer-target hit testing or unrelated page-document behavior changes.

## Constraints

- Do not patch this as a search-screen-only special case if it implies a broader shared interaction-family rule.
- Do not regress the right-pane page/document reading behavior while revisiting the left-pane list behavior.
- Keep the distinction between cursor movement and viewport movement explicit if the list gains both.

## Related

- [View pages and detail presentation](./view-pages-and-details.md)
- [Search interaction cleanup](./search-interaction-cleanup.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
- [TUI architecture](../../architecture/tui.md)
