# View Pages And Detail Presentation

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

Detail pages currently expose a lot of information, but the presentation is still too static and sometimes difficult to scan. Some information is irrelevant for the current record type, and linked navigation inside detail pages is still limited.

## Desired Outcome

Detail pages should feel intentionally composed for the current record:

- hide sections or fields that are irrelevant to the record type
- improve inlining and grouping of important information
- support linked or backlink-style navigation where it materially improves browsing
- keep external-reference affordances lightweight and readable

## Constraints

- Preserve the shared detail-presenter pathway instead of creating per-screen forks.
- Keep the TUI text-first and avoid turning detail pages into dense unstructured dumps.
- Reuse shared ontology/detail models where possible.

## Notes

This item intentionally focuses on the remaining detail-page work, not on the already-landed AoN search-link foundation.

## Related

- [TUI architecture](../../architecture/tui.md)
