# Incomplete Search Tail In Nested Explorer Lists

Status: done  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

This item is complete. The shared ontology-leaf result path and the shared result reader appear to have closed the earlier deep-explorer count mismatch closely enough that this is no longer active backlog work.

The original scratch note called out this concrete example:

- `Search Semantics > Creature > Metadata Fields > families > ghost`

That path reportedly claimed 50+ records but produced a noticeably shorter stepped-in list. The note also contrasted that behavior with a derived-tag path such as:

- `Search Semantics > Creature > Metadata Fields > derivedTags > scene_role > civic_npc`

which produced a much longer and apparently more complete list.

## Desired Outcome

That landed outcome is:

- concrete explorer leaves open through the shared result-reader path instead of a divergent stepped-in record-list flow
- newer result-window loading behavior no longer appears to truncate the inspectable tail for the cases that motivated this note
- the earlier `families > ghost` mismatch is no longer reproduced in spot checks

## Constraints

- Treat this as a correctness and trust issue, not just a presentation problem.
- Avoid papering over the mismatch with copy if the real issue is that one path truncates or filters differently than the other.
- Keep derived-tag and non-derived-tag explorer paths aligned where they are supposed to use the same shared result-opening behavior.

## Notes

The original symptom was a trust issue between explorer counts and the opened result surface, not just a UI-copy problem.

This item is being closed based on spot verification after the shared ontology-leaf and result-reader changes landed. If the mismatch is observed again, reopen it as a focused regression rather than treating the broader explorer/result path as still missing.

## Related

- [TUI architecture](../../../architecture/node/tui.md)
- [Search architecture](../../../architecture/node/search.md)
