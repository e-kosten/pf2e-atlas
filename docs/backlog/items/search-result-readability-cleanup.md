# Search Result Readability Cleanup

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

Some result and breadcrumb lines are difficult to scan because too much metadata is compressed into one line and the formatting does not create clear visual hierarchy.

The original scratch note captured this with examples like:

- `Browse | Creature | Alphabetical: Blue Dragon (Ancient, Spellcaster) | ...`
- `Search Semantics > Creature > Metadata Fields > size: grg | ...`

The issue is not that the data is wrong. It is that the current text layout makes long headers and labels hard to parse quickly.

## Desired Outcome

Tighten result-row and breadcrumb formatting so important information is easier to pick out at a glance.

That work should:

- improve separation between primary record identity and trailing metadata
- reduce cramped or awkwardly wrapped list labels
- keep breadcrumbs readable when deep navigation paths and result metadata collide

## Constraints

- Keep the TUI text-first and compact; do not solve this by turning result rows into multi-line dumps everywhere.
- Preserve the shared presentation pathways instead of creating feature-local formatting forks.
- Favor formatting changes that generalize across record categories.

## Notes

This item is about readability and hierarchy, not about changing the underlying sort or result semantics.

## Related

- [TUI architecture](../../architecture/tui.md)
- [View pages and detail presentation](./view-pages-and-details.md)
