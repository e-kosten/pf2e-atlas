# Remove `isUnique` Metadata + Filter Shape Convergence: Review

## Summary

Taken as a combined execution contract, the April 22 plan plus the April 23 alignment addendum is now aligned with the current architecture and the accepted ADRs for the search-contract refactor. The two substantive gaps from the initial review were resolved by tightening `Slice C` around domain-vs-adapter ownership and `Slice G` around the durable TUI search-facade boundary. After one final wording cleanup in `Slice C` to keep discovery-contract language presentation-neutral, I do not see any additional plan changes that are necessary before implementation begins.

## Review Scope

- Target plan: `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md`
- Alignment addendum: `scratch/plans/2026-04-23-remove-isunique-filter-shape-convergence-alignment-addendum.md`
- Review date: `2026-04-23`
- Review goal: check whether the combined plan aligns with the intended application architecture aside from the already-agreed architectural changes encoded in the addendum and recent ADRs

## Docs And Code Owners Consulted

- `docs/architecture/overview.md`
- `docs/architecture/boundaries.md`
- `docs/architecture/search.md`
- `docs/architecture/tui.md`
- `docs/architecture/decisions/0006-shared-tui-interaction-contracts.md`
- `docs/architecture/decisions/0009-shared-list-detail-presentation-layer.md`
- `docs/architecture/decisions/0010-shared-search-request-contract.md`
- `docs/architecture/decisions/0011-canonical-search-request-and-filter-tree-model.md`
- `docs/architecture/decisions/0012-app-facing-discovery-service-boundary.md`
- `docs/architecture/decisions/0013-tui-canonical-search-state-and-derived-editor-projections.md`
- `docs/architecture/decisions/0014-shared-search-result-presentation-capabilities.md`
- `docs/backlog/items/remove-isunique-metadata.md`
- `docs/backlog/items/filter-shape-convergence.md`
- `docs/backlog/history/items/structured-query-summary-model.md`
- `src/domain/search-request-types.ts`
- `src/tui/search/service.ts`
- `src/tui/search/query-state.ts`
- `src/app/ontology/search-semantics-domain.ts`

## Findings Checklist

- [x] Separate canonical operator vocabulary from edge-sugar parsing ownership
- [x] Make `src/tui/search/service.ts` preservation explicit in the TUI adoption slices

## Detailed Findings

### 1. Separate canonical operator vocabulary from edge-sugar parsing ownership

Why it matters:

- ADR 0011 says friendly or shorthand input is allowed only at the transport/editor edge before lowering into the canonical model.
- The domain boundary requires `src/domain/` to stay transport-agnostic and UI-agnostic.
- The current `Slice C` wording risks putting editor-facing shorthand parsing into the same owner as the canonical operator vocabulary, which would blur the domain boundary the plan is otherwise trying to strengthen.

Evidence:

- `docs/architecture/decisions/0011-canonical-search-request-and-filter-tree-model.md`
- `docs/architecture/boundaries.md`
- `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md`
  - `Slice C` says to introduce one shared owner for canonical operator types and surface-to-canonical operator normalization.
  - The same slice then places the new owner under `src/domain/` and routes operator alias normalization through that owner.

Recommended tightening:

- Keep canonical operator domains, declared field domains, and canonical matcher/value semantics in `src/domain/**`.
- Keep shorthand parsing and alias normalization for editor/transport sugar above the domain layer, for example in server/TUI adapters or a narrowly owned edge-normalization helper that consumes the domain vocabulary.
- Rewrite `Slice C` so it distinguishes:
  - canonical operator/value vocabulary ownership
  - edge sugar parsing ownership
- Preserve the plan’s good rule that sugar lowers once into canonical tokens, but do not make `src/domain/**` the owner of raw user-facing shorthand syntax like `>=` or `1-5`.

Resolution:

- Tightened `Slice C` in `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md` to split canonical domain ownership from adapter-layer sugar parsing. The plan now keeps canonical operator/value semantics in `src/domain/**` and assigns shorthand or alias parsing to MCP/TUI adapter seams or a narrow edge-normalization helper consumed by those seams.

### 2. Make `src/tui/search/service.ts` preservation explicit in the TUI adoption slices

Why it matters:

- The TUI architecture still assigns `src/tui/search/service.ts` as the TUI-facing search facade for query/session behavior, option discovery, ontology-query conversion, count paths, result-window access, and session disposal.
- The combined plan correctly protects the shared discovery boundary through `src/tui/app-services.ts`, but it is less explicit about the rest of the search workflow ownership during `Slice G`.
- Without a sharper owner rule, the implementation could land the canonical `SearchRequest` refactor by moving service responsibilities directly into `search-screen/**`, which would reopen the exact feature-local wiring drift the TUI boundary docs are trying to prevent.

Evidence:

- `docs/architecture/tui.md`
  - the App Services layer keeps feature access narrow through `services.user.*`
  - the Search Service layer names `src/tui/search/service.ts` as the TUI-facing search facade
- `docs/architecture/boundaries.md`
  - TUI feature code should consume explicit facades such as `src/tui/app-services.ts` and `src/tui/search/service.ts`
- `src/tui/search/service.ts`
  - currently owns query normalization, option assembly, count-path use, backend search-window calls, and session disposal
- `scratch/plans/2026-04-23-remove-isunique-filter-shape-convergence-alignment-addendum.md`
  - says new discovery/query-editing behavior should route through current app/search/TUI owners instead of surface-local side channels
- `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md`
  - names `src/tui/search/service.ts` in `Slice G` but does not explicitly state whether the facade remains the durable owner after the canonical-state cutover

Recommended tightening:

- Add an explicit `Slice G` rule that canonical-state adoption does not bypass the TUI search facade.
- Require search-screen controllers and workflows to continue consuming `services.user.search` for:
  - canonical query normalization or helper entrypoints that survive the cutover
  - count/result-window/session operations
  - ontology-origin query conversion
  - search-mode sort and execution behavior
- If the intended end state is to replace `src/tui/search/service.ts` with a different facade owner, say that directly in the plan and update the architecture docs in the same slice. Do not leave the owner ambiguous.

Resolution:

- Tightened `Slice G` in `scratch/plans/2026-04-22-remove-isunique-filter-shape-convergence-plan.md` to require one explicit TUI-facing search facade through `services.user.search` while allowing the implementation behind that facade to narrow or split into richer TUI search owners. The plan now also rules out feature-local `search-screen/**` code silently becoming the owner of workflow/session responsibilities.

## Open Questions Or Non-Issues

- The April 23 addendum already resolves the plan’s earlier biggest TUI architecture risks:
  - modal work extends `src/tui/framework/**` rather than creating a search-local overlay stack
  - picker filtering extends the shared prompt family rather than adding a search-local picker host
  - result grouping and lightweight metadata extend the shared list/detail/result-formatting owners rather than creating a parallel result-view framework
  - the tree and summary remain derived presentation over canonical `SearchRequest`, which is aligned with ADR 0013 and the structured-query-summary history note
- `Slice C1` is directionally aligned with ADR 0012. The remaining concern is mostly owner clarity, not the decision to add the shared app-facing discovery service.
- `Slice H` and `Slice I` are acceptable as long as the addendum remains part of the execution contract and continues to treat lookup as only a consumer of shared result-presentation hooks.

## Recommended Plan Tightenings

- The earlier tightening recommendations have been applied in the plan.
- No further substantive plan changes are recommended before implementation.
