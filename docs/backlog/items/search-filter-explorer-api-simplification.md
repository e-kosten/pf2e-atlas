# Search Filter Explorer API Simplification

Status: proposed  
Priority: later  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

The current search/filter-explorer model is architecturally correct after the boundary-restoration cleanup, but the public TUI search service surface still exposes overlapping draft helpers that are heavier than they need to be.

In particular, the shared search service currently exposes both:

- plain draft creation helpers
- prepared draft helpers that return `{ draft, preservedMetadata, scopedFields }`

Relevant current surfaces:

- `src/tui/search/service-types.ts`
- `src/tui/search/service.ts`
- `src/tui/filter-explorer/search-draft.ts`

That layering is workable, but it leaves the abstraction slightly muddier than the underlying ownership model:

- callers can ask for either a draft or a prepared bundle
- preserved metadata and scope context are still carried through a parallel helper shape instead of one clearly preferred API
- some helper names still reflect the earlier transition away from compatibility draft fields rather than the final durable seam

This is not a live bug. It is follow-on cleanup to make the abstraction layer simpler and easier to use consistently.

## Desired Outcome

Simplify the shared TUI search/filter-explorer API so callers have one clear way to move between query state, compose-draft state, and any required workflow context.

A future implementation should aim for:

- one clearly preferred service surface for filter-explorer draft preparation
- clearer naming for any remaining distinction between durable draft state and workflow/session context
- fewer overlapping helper pairs that differ only in whether they also carry preserved metadata and scope
- call sites that read as if they are using one intentional abstraction rather than choosing between transitional convenience variants

The important goal is to reduce API ambiguity, not to hide real distinctions that callers genuinely need.

## Constraints

- Preserve the architectural outcome from [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md). Do not reintroduce compatibility-only draft fields just to make the API look smaller.
- Do not fold workflow/session concerns back into the durable draft shape.
- Avoid broad search-screen refactors unless they are directly required to simplify the service boundary.
- If both a plain draft and a richer prepared result still need to exist, make the distinction explicit and durable rather than leaving two near-duplicate convenience paths.

## Notes

### Current API shape

The current TUI search service exposes both draft-only and prepared-draft entrypoints:

- `prepareFilterExplorerDraft`
- `prepareFilterExplorerDraftFromMetadataNode`
- `createFilterExplorerDraft`
- `createFilterExplorerDraftFromMetadataNode`

The underlying type surface also still exposes:

- `Pf2eTerminalFilterExplorerDraft`
- `Pf2eTerminalPreparedFilterExplorerDraft`

That shape is understandable during a cleanup, but it is still a larger API than the current ownership model really wants.

### Why the cleanup matters

This matters because service boundaries set expectations for future callers.

If the current overlap remains in place indefinitely, new code will keep choosing among near-duplicate helpers and may start depending on whichever one feels convenient locally. That makes the abstraction harder to teach, harder to narrow with lint rules later, and harder to simplify once more search-screen or explorer-hosting code accumulates around it.

This item is therefore about keeping the now-correct abstraction honest and easy to consume.

### Implementation questions a future task should settle

A future implementation should decide things such as:

- should callers always start from a prepared bundle and pull the `draft` field when they only need the draft?
- should there be one canonical preparation entrypoint plus small local helpers for projections?
- should workflow/session context be modeled with a more explicit name than "prepared draft" if that bundle remains the durable seam?

The specific answer matters less than having one answer that the call sites consistently follow.

### Candidate starting points

- `src/tui/search/service-types.ts`
- `src/tui/search/service.ts`
- `src/tui/filter-explorer/search-draft.ts`
- `src/tui/search-screen/filter-explorer-workflow.ts`
- `src/tui/search-screen/query-field-editing.ts`
- `tests/tui/search-screen.test.tsx`

### Suggested validation checks for a future implementation

- verify that all current callers route through the simplified preferred API shape
- verify that preserved metadata and scoped-field context still survive round-trips where needed
- verify that compose-mode flows and metadata-node-based launch flows still behave identically
- verify that no redundant helper pairs remain unless they encode a real durable distinction

## Related

- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
- [Structured query summary model](./structured-query-summary-model.md)
- [Filter explorer internal decomposition](./filter-explorer-internal-decomposition.md)
- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
