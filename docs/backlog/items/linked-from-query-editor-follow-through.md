# LinkedFrom Query Editor Follow-Through

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-05-03

## Problem

The shared search contract needs `linkedFrom` support so entity-page `Referenced By` drills can compile to ordinary canonical browse requests. However, the current entity-page work does not require full user-facing query-editor or general TUI authoring support for `linkedFrom`.

If that follow-through is left implicit, future implementation work can quietly expand the entity-page slice into broader query-editor, formatting, and explorer UI work that is not required to make page references and backlinks function.

## Desired Outcome

Track the optional follow-through for exposing `linkedFrom` more broadly after the canonical contract support is landed.

That later pass can include:

- query-editor authoring support for `linkedFrom`
- shared TUI query-core formatting and summary support for inbound-link filters
- any explorer or picker affordances that should let users construct `linkedFrom` filters directly
- shared help, labels, and presentation polish for inbound-link search filters beyond page-seeded requests

## Constraints

- Do not block the current entity-page and backlink drill work on this item.
- Keep the canonical contract, normalization, SQL/runtime, and schema support for `linkedFrom` in the active implementation plan; this item is only for broader user-facing editing and presentation follow-through.
- Preserve `SearchRequest` as the durable shared boundary; any editor affordance should lower cleanly into the canonical filter tree instead of introducing a parallel link-filter model.
- Do not force broad editor exposure if the repo later decides `linkedFrom` should stay mostly programmatic or niche.

## Notes

This item exists because the entity-page plan only needs seeded `linkedFrom` requests to power grouped `Referenced By` surfaces. General query-editor support is useful, but it is separate scope.

## Related

- [View pages and detail presentation](./view-pages-and-details.md)
- [Search runtime architecture](../../architecture/search.md)
- [Shared TUI interaction family contracts](./shared-tui-interaction-family-contracts.md)
