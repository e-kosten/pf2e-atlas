# Search Interaction Cleanup

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-20

## Problem

Search interaction is inconsistent across the TUI. `/` filtering is missing from places where it should help narrow large option sets, query text does not always collapse obvious non-matches, and the `:` command palette duplicates controls that are already visible in the editor.

## Desired Outcome

The search editor and related pickers should have one coherent interaction model:

- `/` filters long visible option sets where local narrowing is the right action
- query text affects the actual search result set in the expected places
- the command palette is removed or reduced where it only duplicates on-screen actions

## Constraints

- Keep the shared TUI interaction model coherent rather than adding one-off screen behavior.
- Do not regress existing direct result-reader and ontology-seeded search flows.
- Prefer shared editor/search framework changes over ad hoc local key handling.

## Notes

This item is a consolidation of several smaller interaction notes that all point at the same user-facing inconsistency.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Search architecture](../../architecture/search.md)
