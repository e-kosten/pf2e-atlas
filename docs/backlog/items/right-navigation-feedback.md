# Right-Navigation Feedback

Status: done  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

This item is complete. Shared list/detail screens now have a transient footer-banner notification seam, and filter-explorer dead-end drill behavior uses that seam instead of falling through to focus toggling.

## Desired Outcome

That landed outcome is:

- failed rightward drill shows a lightweight shared footer notification
- failed drill no longer behaves like an implicit pane-focus change
- the feedback path lives on the shared list/detail presentation seam instead of screen-local footer state

## Constraints

- Keep the feedback lightweight and transient.
- Do not turn ordinary dead-end navigation into a modal or blocking interruption.
- Keep the behavior aligned with the shared TUI interaction model rather than adding one-off screen-local hacks.

## Notes

This item was specifically about clarifying failed “go deeper” intent, not redesigning the broader navigation model.

## Related

- [TUI architecture](../../architecture/tui.md)
