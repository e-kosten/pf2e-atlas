# Mouse Text Selection for Copy/Paste

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-05

## Problem

The terminal UI is gaining richer mouse and trackpad behavior, but it does not yet have a durable model for mouse-based text selection intended for copy/paste.

That creates a separate interaction gap from click targets or wheel scrolling:

- users should be able to drag-select visible text for copying
- selection should not accidentally cross pane, modal, footer, or structured-region boundaries in surprising ways
- text selection needs to coexist with click-to-focus, wheel scrolling, and future pointer target hit testing

Without an explicit backlog item, this work is easy to conflate with pointer hit testing even though the behavior and success criteria are different.

## Desired Outcome

Add mouse-based text selection support for copy/paste-oriented workflows in the TUI.

The follow-up should cover:

- a shared selection model for drag start, drag update, and drag end over rendered text
- boundary-aware selection rules for panes, modals, footers, inline regions, and structured detail surfaces
- predictable behavior when a drag begins in one boundary and moves outside it
- copied text normalization for wrapped lines, clipped content, and visible-only regions
- interaction precedence between drag selection, click-to-focus, pointer target activation, and wheel scrolling
- tests or fixtures that lock down boundary behavior for representative TUI surfaces

## Constraints

- Do not treat copy/paste text selection as the same problem as click hit testing.
- Do not add feature-local drag-selection code to individual screens if the behavior belongs in shared TUI interaction infrastructure.
- Preserve normal terminal selection expectations where possible, but make application-owned boundaries explicit when the app intercepts mouse input.
- Keep help/footer copy accurate if text selection introduces visible interaction affordances.

## Notes

This item is adjacent to pointer target hit testing, but it is intentionally separate:

- pointer target hit testing answers "what command target did the user click?"
- text selection answers "what visible text range did the user drag over for copying?"

The implementation should decide whether the app owns copied text directly or only shapes terminal-native selection behavior through mouse reporting and boundary-aware event handling.

## Related

- [Pointer target hit testing](./pointer-target-hit-testing.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
- [Search result list viewport behavior](./search-result-list-viewport-behavior.md)
- [TUI architecture](../../architecture/node/tui.md)
