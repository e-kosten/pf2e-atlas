# Search Page Navigation Shaping

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-04

## Problem

The search page navigation is workable, but the current interaction model does not yet feel like the intended final product. The implementation supports editing, running, reading results, previewing pages, and moving between panes, but the product direction for how those pieces should feel together still needs more deliberate shaping.

The risk is that future fixes keep polishing local behaviors without first deciding the desired navigation model for the search page as a whole.

## Desired Outcome

Define and then implement a more intentional search-page navigation model:

- decide how users should move between query editing, result lists, previews, and opened pages
- decide which actions should be pane focus changes, structural descent/ascent, result execution, page navigation, or action-rail selection
- make the keybinding/help/footer model reflect that product intent clearly
- preserve useful existing behavior while replacing awkward or accidental interaction patterns
- keep the result-reader and page preview behavior aligned with the shared cursor-vs-viewport interaction family

## Constraints

- Start with product/interaction shaping before implementation.
- Keep the backlog item generic until concrete behavior decisions are made.
- Do not regress existing search execution, ontology-origin search, or prepared result-reader flows.
- Prefer shared interaction contracts and route preparation over screen-local key handling.

## Notes

This item intentionally does not prescribe a specific fix. It exists because the current search page works mechanically but still needs a clearer product direction before deeper navigation changes are made.

## Related

- [Search interaction cleanup](./search-interaction-cleanup.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
- [Search result list viewport behavior](./search-result-list-viewport-behavior.md)
- [TUI architecture](../../architecture/tui.md)
