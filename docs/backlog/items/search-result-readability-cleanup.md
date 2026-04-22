# Search Result Readability Cleanup

Status: done  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

This item is complete. Shared list/detail formatting now lives on the shared TUI presentation helpers, with breadcrumbs routed through one formatter and compact default result rows keeping record identity primary while omitting repeated scope text already carried elsewhere in the screen.

## Desired Outcome

That landed outcome is:

- primary record identity leads the shared default result row
- repeated category or subcategory scope no longer gets restated in result rows when the screen subtitle already carries it
- breadcrumbs route through one shared formatter instead of per-screen joins
- shared ontology/search labels and fallback humanization replace raw metadata ids and raw field-type wording in the affected list/detail surfaces

## Constraints

- Keep the TUI text-first and compact; do not solve this by turning result rows into multi-line dumps everywhere.
- Preserve the shared presentation pathways instead of creating feature-local formatting forks.
- Favor formatting changes that generalize across record categories.

## Notes

This item was about readability and hierarchy, not about changing the underlying sort or result semantics.

## Related

- [TUI architecture](../../architecture/tui.md)
- [View pages and detail presentation](./view-pages-and-details.md)
