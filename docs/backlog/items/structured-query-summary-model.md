# Structured Query Summary Model

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

A remaining dirty worktree, `feat/search-structured-staging-worker-d`, preserved a useful design direction for the search editor but an obsolete implementation path.

That scratch work targeted:

- `src/tui/search-screen-workspace.ts`
- `src/tui/search-service.ts`

The current codebase has since moved the workspace code into the split `src/tui/search-screen/workspace/` tree, so the patch is not directly mergeable. The important idea that still remains open on `main` is:

- the structured search query should be representable as a first-class summarized document/model, not only as ad hoc visible workspace rows derived directly from current query state

Today, `main` builds search editor rows more directly from query state and metadata flattening in `src/tui/search-screen/workspace/workspace.ts`. That is workable for the current editor, but it makes future rendering, stable targeting, and alternate presentations harder than they need to be.

The canonical state-cleanup prerequisites from the later `fix/tui-cleanup-slice` direction are now largely landed, which narrows the remaining work:

- `query.filters.parts` is already the canonical TUI structured-query representation
- search/filter-explorer state cleanup is already tracked as landed historical context rather than open architectural work
- the remaining gap is the live workspace/document layer, not the old canonicalization work

## Desired Outcome

Introduce a durable structured-query summary model in the current search-screen architecture.

That future implementation should provide:

- a first-class summary representation for the current structured query
- stable row/section identity for major structured-query concepts such as profile, category, subcategory, level range, rarity, action cost, and metadata clauses
- explicit anchors for metadata nodes and nested query structure
- a clean seam between:
  - query-state interpretation
  - summary/document modeling
  - visible editor row rendering

The summary model should be useful both for the current search editor and for future document-style rendering work, and it should build on the now-landed canonical state ownership instead of recreating transitional compatibility layers.

## Constraints

- Do not revive the old `src/tui/search-screen-workspace.ts` file layout. Work should start from the current `src/tui/search-screen/workspace/` modules and `src/tui/search/` query-state/service seams.
- Treat the dirty worktree as design input, not as a patch to replay.
- Keep query-state ownership and normalization in the current search-owned seams rather than scattering new ad hoc helpers across unrelated files.
- Keep the summary model aligned with the canonical query-parts direction. If existing TUI search state still carries duplicate compatibility fields, clean that state shape first or in the same task rather than teaching the new summary model to depend on both representations.
- Do not introduce abstraction for its own sake. The model should only be added if it clearly improves rendering, navigation, or state targeting in the editor.
- Keep the search editor aligned with the shared TUI interaction and list/detail presentation architecture rather than creating a second bespoke rendering path.

## Notes

### Source context

This item preserves the useful design intent from the dirty worktree `feat/search-structured-staging-worker-d`.

The main useful idea from that worktree was a separate structured-query summary layer with concepts like:

- `SearchStructuredSummaryRow`
- stable anchors such as:
  - `profile`
  - `category`
  - `subcategory`
  - `levelRange`
  - `rarity`
  - `actionCost`
  - `metadata:<path>`
- helper functions to derive a summarized structured-query view before converting it into screen rows

### Supporting cleanup direction

The later dirty worktree `fix/tui-cleanup-slice` preserved a useful prerequisite direction, and that prerequisite is now largely landed:

- keep `query.filters.parts` as the only canonical structured-query representation on the TUI side
- stop carrying extra TUI-only compatibility fields when the same information can be derived from query parts
- reduce the compatibility gap between search query state and filter-explorer compose state

That direction is now tracked as done historical context in [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md). What remains here is the missing summary/document layer on top of that canonical state.

### Why the idea matters

This model is valuable because it would make the search query easier to treat as a document rather than only as a live list of editor rows.

That helps with future work such as:

- richer structured-query detail panes
- stable targeting of logical sections during keyboard navigation
- clearer inactive placeholders and optional sections
- alternate renderers for the same query state
- future document-style query rendering without coupling presentation directly to low-level query-state extraction

In other words, this is less about the current visible search editor polish and more about introducing a cleaner intermediate model that future rendering and navigation work can build on.

### Why the current implementation should not be merged as-is

- It targets old file locations that no longer match `main`.
- Parts of the current search editor have already moved into a newer split architecture.
- The patch would need manual reinterpretation into the current module tree rather than a direct merge.

### Implementation intent

A future implementation should likely start by deciding where the summary model belongs:

- query-state helpers in `src/tui/search/` should continue to own normalized query interpretation
- search-screen workspace modules should own editor-facing summary-to-row rendering
- the summary layer should sit between those two concerns

Do not re-open the already-landed canonical state cleanup inside renderer code. The summary layer should sit on top of the existing canonical state instead of becoming another compatibility owner.

One likely shape is:

1. derive a normalized structured-query snapshot from current query state
2. derive a stable summary/document model from that snapshot
3. render editor rows, detail lines, and focus/help state from that summary model

### Comparison to the current implementation on `main`

Current `main` is simpler and more direct:

- build workspace rows directly from query state
- flatten metadata as needed
- show optional rows only when they are currently active

The deferred work from this item would make the editor more model-driven:

- explicit summary rows
- stable anchors
- cleaner separation between meaning and rendering

That is probably overkill for a first-pass editor, but it becomes valuable once the query editor needs more document-like rendering and stable section-level behavior.

### Suggested validation checks for a future implementation

- verify that structured-query sections have stable identities even when visible row order changes
- verify that metadata clauses can be addressed by stable anchors instead of only by visible row position
- verify that current search editor behavior does not regress while the new model is introduced
- add focused tests around summary derivation separately from screen rendering where practical

## Related

- [Backlog: Search interaction cleanup](../backlog.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [Search filter explorer draft canonicalization](./search-filter-explorer-draft-canonicalization.md)
- [View pages and detail presentation](./view-pages-and-details.md)
- [TUI architecture](../../architecture/tui.md)
- [Search architecture](../../architecture/search.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [ADR 0006: Shared TUI interaction contracts](../../architecture/decisions/0006-shared-tui-interaction-contracts.md)
- [ADR 0009: Shared list/detail presentation layer](../../architecture/decisions/0009-shared-list-detail-presentation-layer.md)
