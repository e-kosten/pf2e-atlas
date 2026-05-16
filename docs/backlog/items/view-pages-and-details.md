# View Pages And Detail Presentation

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

Detail pages currently expose a lot of information, but the presentation is still too static and sometimes difficult to scan. Some information is irrelevant for the current record type, and linked navigation inside detail pages is still limited.

## Desired Outcome

Detail pages should feel intentionally composed for the current record through one shared structured page-document pathway:

- hide sections or fields that are irrelevant to the record type
- improve inlining and grouping of important information
- support linked or backlink-style navigation where it materially improves browsing
- keep external-reference affordances lightweight and readable
- keep page composition on the shared entity-page service seam instead of per-screen record-to-lines assembly
- support section-first reading with explicit section-local target entry for interactive page targets

## Constraints

- Preserve the shared entity-page and page-document pathways instead of creating per-screen forks.
- Keep the TUI text-first and avoid turning detail pages into dense unstructured dumps.
- Reuse shared ontology/detail models where possible.

## Notes

This item intentionally focuses on the remaining detail-page work, not on the already-landed AoN search-link foundation.

## Related

- [TUI architecture](../../architecture/node/tui.md)
