# Backlog Done / Superseded

This index tracks backlog items that are no longer open work.

Use the live [backlog](../backlog.md) for active future work. Keep this file for durable visibility into what has already landed or what has been retired as a standalone item.

## Done

- `Add loading affordances`
  Shared route-transition loading affordances already exist. Remaining work is tuning and polish, not first-pass implementation. Status: done.

- `Terminal UI architecture`
  The old monolithic `terminal-ui.tsx` concern has already been split into shared framework modules. Remaining work is ordinary maintenance, not the original refactor target. Status: done.

- `Terminal modal layout tuning`
  The shared planner is already in place. What remains is product tuning, not missing architecture. Status: done.

- [Search filter explorer draft canonicalization](../items/search-filter-explorer-draft-canonicalization.md)
  Canonical draft/state ownership now lives on the shared search/filter-explorer path instead of compatibility draft fields. Status: done.

- [Filter explorer internal decomposition](../items/filter-explorer-internal-decomposition.md)
  The shared filter explorer now has explicit controller-state, inspect/open, route-handling, draft-query, and draft-model owners instead of relying on `controller.ts` and `search-draft.ts` as subsystem sinks. Status: done.

- [Search filter explorer API simplification](../items/search-filter-explorer-api-simplification.md)
  The TUI search service now exposes one canonical filter-explorer preparation seam instead of overlapping draft-only and prepared-draft helper pairs. Status: done.

- [Structured query summary model](../items/structured-query-summary-model.md)
  The live search workspace now derives one summary/document model from canonical query state and reuses it across workspace rows, staged summaries, and query-status detail rendering. Status: done.

- [Search screen interaction follow-through](../items/search-screen-interaction-follow-through.md)
  The typed search-screen interaction seam, shared help/footer derivation, and unavailable-command palette behavior are now landed in the split search-screen architecture. Status: done.

- [Actor metrics search orchestration](../items/actor-metrics-search-orchestration.md)
  Actor and item metric predicates now run through the shared search/filter pipeline and are wired through both MCP semantics and TUI compose flows. Status: done.

- [Derived-tag ontology future shape](../items/derived-tag-ontology-future-shape.md)
  The durable documentation-preservation goal is now complete; active implementation follow-through is tracked separately under the derived-tag concept-model implementation backlog item. Status: done.

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
