# Backlog

This is the tracked backlog for durable future work.

## Status Vocabulary

- `proposed`
- `planned`
- `in_progress`
- `blocked`
- `deferred`
- `done`
- `superseded`

## Now

- [Search interaction cleanup](./items/search-interaction-cleanup.md)
  Unify how `/`, query text, and action menus work in the search editor and long selection lists. Status: proposed.

- [Search screen interaction follow-through](./items/search-screen-interaction-follow-through.md)
  Preserve the useful ideas from the dropped `feat/search-final-state-completion` scratch work without reviving its obsolete file layout. Status: proposed.

- [View pages and detail presentation](./items/view-pages-and-details.md)
  Make record detail views easier to scan, more dynamic by record type, and better suited for linked navigation. Status: proposed.

- `Ontology browser naming friendliness`
  Replace internal-facing labels such as `derivedTags` and other machine-shaped names with natural casing and wording in the explorer UI. Status: proposed.

- `Search result readability cleanup`
  Tighten list and breadcrumb formatting where result headers and long labels are hard to scan in the current TUI layout. Status: proposed.

- `Right-navigation feedback`
  When rightward navigation cannot drill deeper, show a small transient message instead of silently behaving like a layout/focus toggle. Status: proposed.

- `Remove isUnique metadata`
  Stop exposing `isUnique` as standalone metadata where it only duplicates rarity semantics. Status: proposed.

## Soon

- [Derived-tag assignments layout](./items/derived-tag-assignments-layout.md)
  Decide a durable on-disk structure for authored assignments before the assignment corpus grows much larger. Status: proposed.

- [Structured query summary model](./items/structured-query-summary-model.md)
  Preserve and eventually implement a first-class structured-query document model so future search-editor and document-style rendering work does not depend only on ad hoc workspace rows. Status: proposed.

- [Actor metrics search orchestration](./items/actor-metrics-search-orchestration.md)
  Bridge the gap between actor metrics in search semantics and the actual executable search/query path. Status: proposed.

- [Tagging tooling reorganization](./items/tagging-tooling-reorg.md)
  Move editorial/tagging utilities out of the root npm command surface and finish the repo-level naming cleanup around the TUI app. Status: proposed.

- `Incomplete search tail in nested explorer lists`
  Investigate why some deep record lists, such as `families > ghost`, advertise more results than the stepped-in detail view actually shows. Status: proposed.

- `Filter shape convergence`
  Bring MCP and TUI filter modeling back into alignment, especially around rarity and level no longer being treated as special cases. Status: proposed.

## Later

- [Category relevance script](./items/category-relevance-script.md)
  Add tooling to help tagging work happen in coherent batches without forcing one agent or reviewer to keep an entire family/tag space in active memory. Status: proposed.

- `Metadata predicate typing cleanup`
  The shared metadata-predicate spec architecture has already landed, but there may still be a small follow-up opportunity to simplify or tighten generic predicate typing now that the older `fix/metadata-filter-spec-consolidation` worktree is being retired. Status: proposed.

- `Typed seams cleanup`
  Continue focused type-safety work around metadata registry access, prompt result narrowing, matcher adapters, and test fixtures. Status: proposed.

- `Shared UI model boundary enforcement`
  Identify which reusable TUI state/view-model helpers should become mandatory and enforce them once those pathways are stable enough. Status: proposed.

## Done / Superseded

- `Add loading affordances`
  Shared route-transition loading affordances already exist. Remaining work is tuning and polish, not first-pass implementation. Status: done.

- `Loading icon cleanup`
  This is now a narrower polish problem inside the existing loading treatment rather than a standalone missing feature. Status: superseded.

- `Faster boot`
  The original broad note is too vague; any future startup work should be tracked as targeted loading or route-preparation follow-ups. Status: superseded.

- `Derived tags command palette`
  This no longer stands well as a separate initiative. The remaining useful work belongs under broader search/editor interaction cleanup. Status: superseded.

- `Terminal UI architecture`
  The old monolithic `terminal-ui.tsx` concern has already been split into shared framework modules. Remaining work is ordinary maintenance, not the original refactor target. Status: done.

- `Terminal modal layout tuning`
  The shared planner is already in place. What remains is product tuning, not missing architecture. Status: done.

- `Broader TUI mode switcher`
  The main interaction-context unification work already landed, and this no longer needs its own backlog slot unless a new convergent abstraction appears later. Status: superseded.
