# Ontology Browser Naming Friendliness

Status: done  
Priority: now  
Owner: unassigned  
Last reviewed: 2026-04-22

## Problem

This item is complete. Shared ontology/search wording now routes through the domain-owned presentation vocabulary, and uncatalogued values fall back to one shared friendly humanization path instead of surfacing raw ids such as `EnumString`, `derivedTags`, `party_role`, or `characterCreation`.

## Desired Outcome

That landed outcome is:

- shared metadata-field and field-type labels render through one domain owner
- category, subcategory, tag, predicate, breadcrumb, and detail values fall back to friendly rendering when no explicit alias exists
- stable ids and search semantics stay unchanged beneath the presentation layer

## Constraints

- Do not change canonical ids or underlying search semantics just to improve labels.
- Keep naming consistent between the explorer, detail panes, and any seeded query flows.
- Prefer systematic label shaping over one-off local renames where the same pattern appears in multiple places.

## Notes

The original naming cleanup is done. Any future wording work should be a narrower refinement rather than a first-pass friendliness fix.

## Related

- [TUI architecture](../../../architecture/node/tui.md)
- [Search architecture](../../../architecture/node/search.md)
