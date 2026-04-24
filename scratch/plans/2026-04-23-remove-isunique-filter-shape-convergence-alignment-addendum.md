# Remove `isUnique` Metadata + Filter Shape Convergence: Architecture Alignment Addendum

## 1. Summary

This addendum reviews `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md` against the current architecture docs and the live TUI/search abstractions.

Use the April 22 plan as the scope-and-sequencing draft, but treat this addendum as the architectural alignment layer for implementation. The main corrections are:

- extend existing shared TUI modal/prompt infrastructure instead of inventing a parallel floating-dialog stack
- extend existing shared list/detail presentation and formatting seams instead of creating a parallel result-view framework
- keep the search workspace summary/tree as derived presentation over canonical query state, not a second long-lived semantic model
- route new discovery/query-editing behavior through current app/search/TUI owners instead of adding surface-local side channels

## 2. Architecture Context

Read and preserve:

- `docs/architecture/overview.md`
- `docs/architecture/boundaries.md`
- `docs/architecture/search.md`
- `docs/architecture/tui.md`
- `docs/architecture/decisions/0006-shared-tui-interaction-contracts.md`
- `docs/architecture/decisions/0009-shared-list-detail-presentation-layer.md`
- `docs/backlog/history/items/structured-query-summary-model.md`

Current owners that the implementation plan should build on directly:

- `src/domain/search-request-types.ts`
  - current shared semantic contract owner for `SearchRequest`
- `src/server/search-request-adapter.ts`
  - current MCP-to-domain request adaptation seam
- `src/search/request-compilation.ts`
  - current semantic-to-execution lowering seam
- `src/tui/search/service.ts`
  - current TUI-facing search facade boundary
- `src/tui/search/query-state.ts`
  - current owner of query normalization/helpers that should either be replaced directly or narrowed into derived/editor helpers, not bypassed with a third model
- `src/tui/search-screen/workspace/query-summary.ts`
  - current owner of the derived summary/document model
- `src/tui/framework/provider.tsx`
  - current modal host/provider composition root
- `src/tui/framework/types.ts`
  - current terminal modal state and prompt option owner
- `src/tui/framework/modal-planning.ts`
  - current modal layout planning owner
- `src/tui/framework/modal-host.tsx`
  - current modal rendering and input-routing owner
- `src/tui/framework/modal-prompt-bodies.tsx`
  - current shared prompt body owner
- `src/tui/interaction-context-adapters.ts`
  - current prompt-adapter seam for feature workflows
- `src/tui/list-detail-presentation.ts`
  - current shared list/detail screen-model, measurement, and notification owner
- `src/tui/list-detail-behavior.ts`
  - current shared list/detail rightward/dead-end behavior owner
- `src/tui/list-detail-formatting.ts`
  - current shared breadcrumb and default result-row formatting owner
- `src/tui/search-screen/results.ts`
  - current search result row/detail presentation seam built on the shared list/detail layer

## 3. Alignment Findings

### 3.1 Shared modal/prompt stack should be extended, not replaced

The April 22 plan is directionally right that search needs a centered mode-picker experience, but the implementation should not introduce a new standalone dialog abstraction that bypasses the existing modal stack.

Required plan correction:

- centered floating mode selection should be implemented by extending the existing modal framework in `src/tui/framework/*`
- the new capability should fit through existing terminal prompt adapters so search workflows still depend on `prompt*` seams instead of feature-local modal state
- new layout or presentation options should be added to the existing modal types/planning/rendering path, not owned by `src/tui/search-screen/**`

Implication for the original plan:

- `Slice E` should be treated as an extension of the shared modal host, modal layout planner, modal prompt bodies, and prompt adapters
- the mode picker may add a new modal kind or a new prompt presentation mode, but it should not become a search-owned overlay framework

### 3.2 Picker filtering should generalize the current prompt family

The original plan already points in the right direction here. The important architectural constraint is that `/` filtering should extend the current shared picker/prompt family and reuse the ontology-style footer filtering pattern rather than creating separate search-specific picker implementations.

Required plan correction:

- implement picker filtering inside the existing modal host/planning/prompt-body path
- keep filtering behavior reachable through the existing `promptSelectOption` and related prompt adapters when possible
- treat clause-kind pickers and other focused editors as consumers of one shared picker-filter capability

### 3.3 The TUI may adopt the shared union literally without abandoning derived editor models

The April 22 plan says the TUI should adopt the shared discriminated union directly. That is compatible with the current architecture only if the semantic source of truth becomes the shared `SearchRequest` while the workspace tree and summary stay derived presentation models.

Required plan correction:

- keep canonical semantic state as the shared `SearchRequest`
- keep `src/tui/search-screen/workspace/query-summary.ts` as the owner of the derived summary/document layer
- allow a derived tree/editor projection with ephemeral ids and insertion slots, but treat it as presentation state only
- do not replace the current summary/document seam with a second durable semantic query model

Implication for the original plan:

- `Slice G` should explicitly preserve the structured-query-summary-model architecture rule while replacing the underlying semantic contract
- any replacement of `src/tui/search/query-state.ts` should either remove it directly or narrow it to derived/editor helpers; it should not survive as a parallel semantic contract beside the new domain-owned union

### 3.4 Result-view work should extend existing list/detail owners

The current plan proposes a “shared result-view presentation capability.” The repo already has the shared list/detail presentation/behavior/formatting owners for this class of work.

Required plan correction:

- result grouping, row badges/subtitles, and detail metadata should extend the existing search-result and list/detail presentation seams
- shared row formatting should continue to live with `src/tui/list-detail-formatting.ts` and nearby shared result helpers
- grouped-result rendering should remain a consumer of `src/tui/list-detail-presentation.ts`, not a separate parallel screen-model framework

Implication for the original plan:

- `Slice H` should be reframed as an extension of shared list/detail formatting and search-result presentation hooks, not as a new generic result-view layer beside list/detail
- `Slice I` should consume those shared list/detail/result-formatting extensions for lookup-specific grouping and match-strength presentation

### 3.5 Discovery boundary work should build on the current app/search split

The original plan’s shared discovery contract is reasonable, but it should be shaped as an app/search service boundary that both ontology and TUI can consume without moving UI concerns into the domain layer.

Required plan correction:

- keep raw discovery execution behind app/search service owners
- keep browse models, rows, picker copy, and mode-specific UI presentation in TUI or ontology consumers
- avoid introducing a domain-level “discovery UI contract” that mixes transport-neutral semantics with terminal/editor affordances

Implication for the original plan:

- `Slice C1` should land as a shared app-facing discovery service seam consumed by TUI search and ontology flows
- that slice should not move picker rows, badges, footer text, or list/detail concerns below the TUI/app boundary

## 4. Slice Adjustments

Apply these adjustments to the April 22 plan during implementation:

- `Slice E`: rename the working goal from “floating dialog primitive” to “shared modal-framework extension for centered choice dialogs,” and scope it to `src/tui/framework/provider.tsx`, `types.ts`, `modal-planning.ts`, `modal-host.tsx`, `modal-prompt-bodies.tsx`, and prompt adapters.
- `Slice F`: keep it as a shared picker-filtering slice, but require it to extend the existing select/multiselect/policy prompt family instead of introducing a new picker host.
- `Slice G`: keep the “literal shared union” goal, but explicitly preserve the summary/document and derived-tree presentation pattern from `structured-query-summary-model.md`.
- `Slice H`: re-scope it to shared list/detail/result-formatting extensions rather than a new result-view abstraction family.
- `Slice I`: require lookup grouping/badges/detail metadata to plug into the shared list/detail/result-formatting hooks from `Slice H`.

## 5. Validation Additions

In addition to the validation already listed in the April 22 plan, implementation should prove these architectural outcomes:

- no search-owned modal framework appears outside the shared `src/tui/framework/*` modal owners
- mode picker integration goes through existing prompt/modal seams rather than feature-local overlay state
- grouped lookup results still render through the existing shared list/detail presentation path
- shared result row/detail formatting extensions live on current shared formatting/presentation owners instead of lookup-local code
- the TUI keeps exactly one semantic query source of truth after migration, with any tree/summary state remaining derived presentation
- docs stay consistent with ADR 0006, ADR 0009, and the structured-query-summary history note

## 6. Landing Guidance

This is still planning-only work on `main`. Do not commit this addendum by itself unless the user later asks for planning artifacts to be committed.

When implementation starts in a `/tmp` worktree, treat:

- `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md`
- `scratch/plans/2026-04-23-remove-isunique-filter-shape-convergence-alignment-addendum.md`

as the combined execution contract.
