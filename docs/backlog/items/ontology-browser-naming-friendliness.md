# Ontology Browser Naming Friendliness

Status: proposed  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

The ontology and search-semantics explorer still surfaces some machine-shaped names that read naturally in code but not in the TUI. Examples called out in scratch planning included labels like `EnumString` and `derivedTags`.

Those labels make the explorer feel internal-facing instead of reader-friendly, especially in a text UI where wording carries more of the interface burden.

## Desired Outcome

Make the explorer labels read like natural user-facing copy rather than implementation vocabulary.

That work should:

- replace internal casing and type-flavored labels with natural wording
- preserve stable underlying semantics and ids while improving displayed copy
- keep the explorer readable without hiding important distinctions between field types or domains

## Constraints

- Do not change canonical ids or underlying search semantics just to improve labels.
- Keep naming consistent between the explorer, detail panes, and any seeded query flows.
- Prefer systematic label shaping over one-off local renames where the same pattern appears in multiple places.

## Notes

The scratch note that prompted this item specifically called out explorer labels like `EnumString` and `derivedTags` as unfriendly in the TUI.

## Related

- [TUI architecture](../../architecture/tui.md)
- [Search architecture](../../architecture/search.md)
