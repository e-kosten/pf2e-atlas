# Shared Result Grouping And Presentation Modes

Status: deferred  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

The shared list/detail presentation cleanup now covers default result-row readability, breadcrumb formatting, lightweight footer notifications, and lookup-specific tiered/global presentation through the shared grouping seam.

Future grouping work should not grow as one-off experiments inside search, ontology, or review screens. If users need to pivot between grouped and flat presentations, that capability needs one shared owner and one shared vocabulary for the available modes.

## Desired Outcome

Add shared infrastructure for optional grouped or alternative list/detail presentations without reintroducing per-screen formatting forks.

That follow-up should cover:

- grouping keys beyond simple sort order, such as category, subcategory, pack, field type, or other shared result metadata
- user-cyclable presentation or grouping modes where a surface can switch between compact flat results and shared grouped views
- shared state, shared footer/help copy, and shared rendering rules rather than screen-local prototypes

## Constraints

- Keep this work on shared TUI presentation owners instead of adding bespoke grouping logic to one screen.
- Do not regress the compact default presentation that now omits already-known scope text from result rows.
- Preserve the existing search/runtime semantics; this item is about presentation and grouping, not about changing result meaning or backend sort behavior.

## Notes

This item is intentionally deferred from the shared presentation vocabulary and dead-end drill cleanup task. The current default presentation should stay stable while grouping and alternative display modes are designed as reusable infrastructure.

## Related

- [TUI architecture](../../architecture/node/tui.md)
- [Architectural boundaries](../../architecture/node/boundaries.md)
