# Right-Navigation Feedback

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

When the user presses rightward navigation expecting to drill deeper and there is no deeper page to open, the current behavior can look too similar to a layout or focus toggle. That makes the action feel ambiguous and surprising.

The original scratch note called out that this often happens in places where the user reasonably expects another level to exist, so silent fallback behavior is misleading.

## Desired Outcome

When rightward navigation cannot open a deeper destination, show a small transient message that explains what happened instead of failing silently or looking identical to another navigation action.

## Constraints

- Keep the feedback lightweight and transient.
- Do not turn ordinary dead-end navigation into a modal or blocking interruption.
- Keep the behavior aligned with the shared TUI interaction model rather than adding one-off screen-local hacks.

## Notes

This item is specifically about clarifying failed “go deeper” intent, not about redesigning the broader navigation model.

## Related

- [TUI architecture](../../architecture/tui.md)
