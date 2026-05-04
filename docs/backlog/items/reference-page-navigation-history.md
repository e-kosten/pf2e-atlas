# Reference Page Navigation History

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-04

## Problem

Reference navigation is functional but not yet comfortable as a browsing flow. Following a page reference can open a search result route, but that route does not behave like a natural page-to-page navigation stack. Users can end up looking at a background search tab with a single item instead of feeling like they moved from one record page to another record page.

That makes it hard to bounce through outgoing references, backlinks, and inline references while preserving a clear way to get back to the previous page context.

## Desired Outcome

Reference navigation should feel like page browsing with history:

- activating a concrete record reference from a page opens that referenced record as a full-screen entity page
- users should not see an incidental one-result search tab when the intent is to navigate to a specific record
- page-to-page navigation should maintain a back stack so users can move through references and then return through the path they took
- grouped reference or backlink searches should still open result-reader routes when the target is intentionally a result set rather than one concrete record
- the flow should preserve shared route preparation, page-document rendering, and canonical target activation semantics

## Constraints

- Do not replace canonical `SearchRequest` result-set navigation for grouped search pivots.
- Do not add a page-local navigation stack that bypasses the shared app navigation owner.
- Keep record-page route preparation render-ready before commit.
- Preserve the distinction between opening a specific record and opening a seeded result set.

## Notes

This item is a follow-up to the entity-page and reference-navigation work. It should start by shaping the exact user model for page history, record preview/open behavior, and back/return semantics before implementation.

## Related

- [View pages and detail presentation](./view-pages-and-details.md)
- [Search result list viewport behavior](./search-result-list-viewport-behavior.md)
- [TUI architecture](../../architecture/tui.md)
