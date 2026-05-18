# LinkedFrom Query Editor Follow-Through

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

The shared search contract includes `linked_from` as the canonical inverse record-link filter. Entity-page `Referenced By` grouped drills, however, are seeded from page-relation grouping and compile to ordinary canonical browse requests using `links_to(targetRecordKey)`. The current entity-page work does not require full user-facing query-editor or general TUI authoring support for authoring `linked_from` directly.

If that follow-through is left implicit, future implementation work can quietly expand the entity-page slice into broader query-editor, formatting, and explorer UI work that is not required to make page references and backlinks function.

## Desired Outcome

Track the optional follow-through for exposing `linked_from` more broadly after the canonical contract support is landed.

That later pass can include:

- query-editor authoring support for `linked_from`
- shared TUI query-core formatting and summary support for inbound-link filters
- any explorer or picker affordances that should let users construct `linked_from` filters directly
- shared help, labels, and presentation polish for inbound-link search filters beyond page-seeded requests

## Constraints

- Do not block the current entity-page and backlink drill work on this item.
- Keep the canonical contract, normalization, SQL/runtime, and schema support for `linked_from` in the active implementation plan; this item is only for broader user-facing editing and presentation follow-through.
- Preserve `SearchRequest` as the durable shared boundary; any editor affordance should lower cleanly into the canonical filter tree instead of introducing a parallel link-filter model.
- Do not force broad editor exposure if the repo later decides `linked_from` should stay mostly programmatic or niche.

## Notes

This item exists because canonical inverse-link filtering is useful outside the entity-page grouped drill path. General query-editor support for `linked_from` is useful, but it is separate scope from the `Referenced By` page surface, whose grouped drills use `links_to(targetRecordKey)`.

## Related

- [View pages and detail presentation](./view-pages-and-details.md)
- [Search runtime architecture](../../architecture/node/search.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
