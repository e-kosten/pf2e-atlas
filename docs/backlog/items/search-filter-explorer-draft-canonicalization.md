# Search Filter Explorer Draft Canonicalization

Status: proposed  
Priority: soon  
Owner: unassigned  
Last reviewed: 2026-04-21

## Problem

A now-retired dirty worktree, `fix/tui-cleanup-slice`, contained a useful cleanup direction for TUI search state and filter-explorer draft ownership, but the patch itself is not suitable for direct merge.

That scratch work mixed together several concerns, some of which have already landed on `main`:

- shared policy presentation moved into `src/tui/framework/`
- ontology presenter and explorer-cache ownership moved under `src/app/ontology/`
- old TUI-local ontology wrapper files were removed

The remaining useful part of the worktree is narrower:

- `query.filters.parts` should stay the canonical TUI structured-query representation
- TUI search state should not keep parallel compatibility fields for subcategory, level range, rarity, action cost, and metadata when those values can be derived from query parts
- filter-explorer compose state should converge on one durable draft shape instead of carrying side-channel compatibility fields such as `structuredMetadata`
- workflow/session ownership should carry exploration scope such as `scopedFields`, rather than baking that scope into the draft object itself

Current `main` is partway there, but still carries compatibility layers such as `Pf2eTerminalFilterExplorerDraft` and search/filter-explorer seams that split ownership between canonical compose state and search-specific compatibility fields.

## Desired Outcome

Finish the state-shape cleanup between TUI search query state and filter-explorer compose state so the durable owners are clear.

That future implementation should aim for:

- `query.filters.parts` as the sole canonical TUI structured-query representation
- one canonical filter-explorer compose draft shape
- metadata owned directly by the compose draft when compose mode needs it
- workflow/session-level ownership of scope such as `scopedFields`
- no extra compatibility-only draft fields that exist only to bridge older search-screen shapes

The goal is not just type cleanup. The goal is to reduce duplicated state ownership so future rendering and editor work can build on one stable model.

## Constraints

- Do not replay the dropped worktree literally. Several edited paths there no longer match the current split `src/tui/search-screen/` layout.
- Preserve the current architecture where shared policy presentation lives in `src/tui/framework/` and ontology presenter/cache ownership lives under `src/app/ontology/`.
- Keep `query.filters.parts` canonical rather than reintroducing parallel field copies for convenience.
- Do not hide this cleanup inside a renderer-only feature. If summary/document rendering work needs this foundation, make the state cleanup explicit.
- Keep filter-explorer mode boundaries clear: compose state should own compose concerns, while workflow/session objects own launch scope and host-specific context.

## Notes

### Source context

This item preserves the unique useful direction from the retired dirty worktree `fix/tui-cleanup-slice`.

The scratch patch suggested all of the following:

- removing duplicated TUI query fields such as `subcategory`, `levelMin`, `levelMax`, `rarity`, `actionCost`, and `metadata` from the canonical search-query shape in favor of deriving them from query parts
- letting `FilterExplorerComposeDraft` carry `metadata` directly instead of relying on a parallel `structuredMetadata` field
- passing `scopedFields` through workflow/session ownership instead of embedding it in the draft shape
- trimming search/filter-explorer seams that still reflected an older compatibility-oriented model

### Why the idea matters

This cleanup is foundational rather than user-visible on its own.

It matters because it reduces the number of places where the TUI has to keep equivalent state in sync:

- query-state interpretation becomes simpler
- filter-explorer compose mode becomes easier to reason about
- future search-screen rendering work can depend on one model instead of bridging multiple transitional shapes
- type signatures become clearer because they describe one owner shape instead of a compatibility bundle

This is especially relevant to future work on richer structured-query rendering and document-style search editing.

### Why the current implementation should not be merged as-is

- The dirty patch targets several older paths that no longer exist on `main`, especially older search-screen module locations.
- Parts of the cleanup direction have already landed through different commits, so the patch is partly obsolete.
- One part of the scratch work is stale in the wrong direction: its `refresh-index` import move no longer matches the current storage-owner split on `main`.

So the remaining value is in the ownership direction, not in the literal diff.

### Implementation intent

A future implementation should likely proceed in small, explicit steps:

1. identify which remaining TUI search/filter-explorer types are true owners versus compatibility wrappers
2. remove parallel compatibility fields where the same value can be derived from query parts
3. converge compose-mode draft state on one canonical shape
4. move scope-bearing fields such as `scopedFields` onto workflow/session objects where they belong
5. tighten tests around canonical query-state and draft behavior after each reduction

Candidate starting points in the current tree:

- `src/tui/search/query-state.ts`
- `src/tui/search/service-types.ts`
- `src/tui/filter-explorer/search-draft.ts`
- `src/tui/search-screen/filter-explorer-workflow.ts`
- `src/tui/search-screen/filter-explorer-screen.tsx`

### Relationship to other backlog items

This is related to, but distinct from, [Structured query summary model](./structured-query-summary-model.md).

- this item is about canonical state ownership and draft shape
- the structured-query summary item is about introducing a richer intermediate summary/document model for rendering and stable section identity

In practice, this cleanup likely makes that later summary-model work easier and less error-prone.

### Suggested validation checks for a future implementation

- verify that TUI search behavior still round-trips correctly through `query.filters.parts`
- verify that filter-explorer compose state can represent metadata and scalar clauses without side-channel compatibility fields
- verify that scope-bearing data such as `scopedFields` is available where workflows need it without bloating the draft shape
- add focused tests around canonical state transforms rather than relying only on broad end-to-end screen tests

## Related

- [Structured query summary model](./structured-query-summary-model.md)
- [Search screen interaction follow-through](./search-screen-interaction-follow-through.md)
- [TUI architecture](../../architecture/tui.md)
- [Architectural boundaries](../../architecture/boundaries.md)
- [Search architecture](../../architecture/search.md)
