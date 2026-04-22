# Incomplete Search Tail In Nested Explorer Lists

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

Some deep explorer lists advertise more results than the stepped-in record list actually shows.

The original scratch note called out this concrete example:

- `Search Semantics > Creature > Metadata Fields > families > ghost`

That path reportedly claimed 50+ records but produced a noticeably shorter stepped-in list. The note also contrasted that behavior with a derived-tag path such as:

- `Search Semantics > Creature > Metadata Fields > derivedTags > scene_role > civic_npc`

which produced a much longer and apparently more complete list.

## Desired Outcome

Make the count shown in deep explorer leaves and the stepped-in detail/result view agree, or explain the difference if the two surfaces are intentionally measuring different things.

## Constraints

- Treat this as a correctness and trust issue, not just a presentation problem.
- Avoid papering over the mismatch with copy if the real issue is that one path truncates or filters differently than the other.
- Keep derived-tag and non-derived-tag explorer paths aligned where they are supposed to use the same shared result-opening behavior.

## Notes

The key problem is the mismatch between what the explorer claims and what the user can actually inspect after opening that node.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Search architecture](../../architecture/search.md)
