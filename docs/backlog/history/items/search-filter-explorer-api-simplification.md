# Search Filter Explorer API Simplification

Status: done  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

This item is complete. It remains here as durable context for the service-seam cleanup that landed during the filter-explorer boundary-closure refactor on 2026-04-21.

Before that refactor, the shared search service still exposed both:

- plain draft creation helpers
- prepared draft helpers that return `{ draft, preservedMetadata, scopedFields }`

Relevant current surfaces:

- `src/tui/search/service-types.ts`
- `src/tui/search/service.ts`
- `src/tui/filter-explorer/search-draft.ts`

That layering was workable, but it left the abstraction slightly muddier than the underlying ownership model:

- callers can ask for either a draft or a prepared bundle
- preserved metadata and scope context are still carried through a parallel helper shape instead of one clearly preferred API
- some helper names still reflect the earlier transition away from compatibility draft fields rather than the final durable seam

This was not a live bug. It was follow-on cleanup to make the abstraction layer simpler and easier to use consistently.

## Desired Outcome

This outcome is now landed.

The landed service seam now provides:

- one clearly preferred service surface for filter-explorer draft preparation
- fewer overlapping helper pairs that differ only in whether they also carry preserved metadata and scope
- call sites that read as if they are using one intentional abstraction instead of choosing between transitional convenience variants

The important goal was to reduce API ambiguity without folding workflow/session state back into the durable draft shape.

## Constraints

- Preserve the architectural outcome from [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md). Do not reintroduce compatibility-only draft fields just to make the API look smaller.
- Do not fold workflow/session concerns back into the durable draft shape.
- Avoid broad search-screen refactors unless they are directly required to simplify the service boundary.
- If both a plain draft and a richer prepared result still need to exist, make the distinction explicit and durable rather than leaving two near-duplicate convenience paths.

## Notes

### Landed API shape

The overlapping draft-only helper pair was removed from the public TUI search service surface. The canonical seam is now the prepared-draft pathway:

- `prepareFilterExplorerDraft`
- `prepareFilterExplorerDraftFromMetadataNode`

The underlying type surface still exposes:

- `Pf2eTerminalFilterExplorerDraft`
- `Pf2eTerminalPreparedFilterExplorerDraft`

That remaining distinction reflects a real model split between durable compose draft state and workflow/session context, not a duplicate public helper pair.

### Why the cleanup matters

This matters because service boundaries set expectations for future callers.

This cleanup matters because future search-screen and explorer-hosting code now has one obvious preparation path instead of two overlapping public choices.

### Validation that landed with the refactor

- search-service tests cover the canonical preparation seam
- preserved metadata and scoped fields still round-trip through filter-explorer workflows
- compose-mode and metadata-node-based launch flows retained behavior
- full `npm run build`
- full `cd scripts && npm test`

## Related

- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
- [Structured query summary model](./structured-query-summary-model.md)
- [Filter explorer internal decomposition](./filter-explorer-internal-decomposition.md)
- [TUI architecture](../../../architecture/node/tui.md)
- [Architectural boundaries](../../../architecture/node/boundaries.md)
