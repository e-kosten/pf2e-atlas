# Backlog Done / Superseded

This index tracks backlog items that are no longer open work.

Use the live [backlog](../backlog.md) for active future work. Keep this file for durable visibility into what has already landed or what has been retired as a standalone item.

## Done

- [Core architecture convergence](./items/core-architecture-convergence.md)
  The long-term non-tag architecture cleanup from the May 2026 codebase architecture review is landed across data/search retrieval, canonical TUI query editing, typed indexing stages, and search-discovery/ontology ownership. Status: done.

- `Add loading affordances`
  Shared route-transition loading affordances already exist. Remaining work is tuning and polish, not first-pass implementation. Status: done.

- `Terminal UI architecture`
  The old monolithic `terminal-ui.tsx` concern has already been split into shared framework modules. Remaining work is ordinary maintenance, not the original refactor target. Status: done.

- `Terminal modal layout tuning`
  The shared planner is already in place. What remains is product tuning, not missing architecture. Status: done.

- [Search filter explorer draft canonicalization](./items/search-filter-explorer-draft-canonicalization.md)
  Canonical draft/state ownership now lives on the shared search/filter-explorer path instead of compatibility draft fields. Status: done.

- [Filter explorer internal decomposition](./items/filter-explorer-internal-decomposition.md)
  The shared filter explorer now has explicit controller-state, inspect/open, route-handling, draft-query, and draft-model owners instead of relying on `controller.ts` and `search-draft.ts` as subsystem sinks. Status: done.

- [Search filter explorer API simplification](./items/search-filter-explorer-api-simplification.md)
  The TUI search service now exposes one canonical filter-explorer preparation seam instead of overlapping draft-only and prepared-draft helper pairs. Status: done.

- [Structured query summary model](./items/structured-query-summary-model.md)
  The live search workspace now derives one summary/document model from canonical query state and reuses it across workspace rows, staged summaries, and query-status detail rendering. Status: done.

- [Search screen interaction follow-through](./items/search-screen-interaction-follow-through.md)
  The typed search-screen interaction seam, shared help/footer derivation, and unavailable-command palette behavior are now landed in the split search-screen architecture. Status: done.

- [Actor metrics search orchestration](./items/actor-metrics-search-orchestration.md)
  Actor and item metric predicates now run through the shared search/filter pipeline and are wired through both MCP semantics and TUI compose flows. Status: done.

- [Remove isUnique metadata](./items/remove-isunique-metadata.md)
  Search/filter vocabulary no longer exposes `isUnique` as a standalone public metadata concept; rarity remains the durable public owner of uniqueness semantics. Status: done.

- [Filter shape convergence](./items/filter-shape-convergence.md)
  MCP, TUI, and docs now describe the same canonical `SearchRequest` contract on `mode`, optional `search`, and root `filter`, with lookup-specific sort and match-presentation behavior routed through the shared result-view path. Status: done.

- [Derived-tag ontology future shape](./items/derived-tag-ontology-future-shape.md)
  The durable documentation-preservation goal is now complete; active implementation follow-through is tracked separately under the derived-tag concept-model implementation backlog item. Status: done.

- [Right-navigation feedback](./items/right-navigation-feedback.md)
  Failed drill intent on the shared filter explorer now routes through a shared transient footer notification instead of falling through to pane-focus behavior. Status: done.

- [Search result readability cleanup](./items/search-result-readability-cleanup.md)
  Shared breadcrumb formatting and compact default result rows are now part of the shared list/detail presentation path instead of per-screen string assembly. Status: done.

- [Ontology browser naming friendliness](./items/ontology-browser-naming-friendliness.md)
  Shared ontology/search labels and fallback humanization now keep explorer and detail copy friendly without changing canonical ids or query semantics. Status: done.

- [Shared list/detail behavior contracts](./items/shared-list-detail-behavior-contracts.md)
  Rightward list/detail intent, dead-end handling, and explicit-only focus policy now live on a shared behavior contract instead of drifting across the qualifying search and explorer screens. Status: done.

- [Structured draft shared explorer session cleanup](./items/structured-draft-shared-explorer-session-cleanup.md)
  The 2026-05-03 structured-draft cleanup routed grouped and single-clause shared-explorer launches through one intermediate session coordinator without changing their distinct writeback semantics. Status: done.

- [Structured editor continuation model convergence](./items/structured-editor-continuation-model-convergence.md)
  Structured search-editor child flows now share a search-host-owned continuation model over canonical `SearchRequest` state, with explicit resume-target ownership and bounded mutation semantics. Status: done.

- [Structured editor SRP decomposition](./items/structured-editor-srp-decomposition.md)
  Structured search-editor actions now split host mutations, grouped field setup, explorer flows, prompt flows, structural routing, and React composition into separate owners instead of routing through one broad metadata-action hook. Status: done.

- [Structured editor routing layer](./items/structured-editor-routing-layer.md)
  Structured query field edits now route through explicit grouped-field and leaf owners before prompt, explorer, or writeback behavior runs. Status: done.

- [Incomplete search tail in nested explorer lists](./items/incomplete-search-tail.md)
  The shared ontology leaf and result-reader path now appears to keep deep explorer counts aligned closely enough in practice that this is no longer active backlog work. Status: done.

- `Concern-specific tag facades`
  The runtime, editorial, and editorial-UI facades now match the intended concern split instead of acting like a mixed catch-all surface. Status: done.

- `Editorial index-opening boundary`
  Short-lived app/editorial index access now routes through the shared application storage owner instead of parallel helper seams. Status: done.

## Superseded

- `Loading icon cleanup`
  This is now a narrower polish problem inside the existing loading treatment rather than a standalone missing feature. Status: superseded.

- `Faster boot`
  The original broad note is too vague; any future startup work should be tracked as targeted loading or route-preparation follow-ups. Status: superseded.

- `Derived tags command palette`
  This no longer stands well as a separate initiative. The remaining useful work belongs under broader search/editor interaction cleanup. Status: superseded.

- `Broader TUI mode switcher`
  The main interaction-context unification work already landed, and this no longer needs its own backlog slot unless a new convergent abstraction appears later. Status: superseded.
